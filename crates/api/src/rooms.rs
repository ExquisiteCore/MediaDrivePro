use axum::{
    Router,
    extract::{Path, Query, State, WebSocketUpgrade, ws::{Message, WebSocket}},
    response::IntoResponse,
    routing::{delete, get, post},
};
use futures::{SinkExt, StreamExt};
use mdp_auth::middleware::AuthUser;
use mdp_common::error::AppError;
use mdp_common::response::ApiResponse;
use mdp_core::room::{MemberInfo, RoomDetail, RoomInfo, RoomService};
use sea_orm::EntityTrait;
use serde::Deserialize;
use uuid::Uuid;

use crate::room_manager::*;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/rooms", post(create_room))
        .route("/rooms", get(list_rooms))
        .route("/rooms/join", post(join_room))
        .route("/rooms/{id}", get(get_room))
        .route("/rooms/{id}", delete(close_room))
        .route("/rooms/{id}/play", post(set_playing))
        .route("/rooms/{id}/members", get(list_members))
}

pub fn ws_routes() -> Router<AppState> {
    Router::new().route("/rooms/{id}/ws", get(ws_upgrade))
}

// --- REST handlers ---

#[derive(Deserialize)]
struct CreateRoomReq {
    name: String,
    max_members: Option<i32>,
}

async fn create_room(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::Json(body): axum::Json<CreateRoomReq>,
) -> Result<ApiResponse<RoomInfo>, AppError> {
    let room = RoomService::create(&state.db, auth.user_id, &body.name, body.max_members).await?;
    Ok(ApiResponse::ok(room))
}

async fn list_rooms(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<ApiResponse<Vec<RoomInfo>>, AppError> {
    let rooms = RoomService::list_by_user(&state.db, auth.user_id).await?;
    Ok(ApiResponse::ok(rooms))
}

async fn get_room(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<RoomDetail>, AppError> {
    if !RoomService::is_member(&state.db, id, auth.user_id).await? {
        return Err(AppError::Forbidden);
    }
    let detail = RoomService::get_detail(&state.db, id).await?;
    Ok(ApiResponse::ok(detail))
}

async fn close_room(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<()>, AppError> {
    RoomService::close(&state.db, id, auth.user_id).await?;

    if let Some(tx) = state.room_channels.get(&id) {
        let msg = WsOut::Error {
            code: "ROOM_CLOSED".to_string(),
            message: "房间已关闭".to_string(),
        };
        let _ = tx.send(serde_json::to_string(&msg).unwrap_or_default());
    }
    cleanup_channel(&state.room_channels, id);

    Ok(ApiResponse::ok(()))
}

#[derive(Deserialize)]
struct JoinReq {
    invite_code: String,
}

async fn join_room(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::Json(body): axum::Json<JoinReq>,
) -> Result<ApiResponse<RoomInfo>, AppError> {
    let room = RoomService::get_by_invite(&state.db, &body.invite_code).await?;
    RoomService::join(&state.db, room.id, auth.user_id).await?;
    let room = RoomService::get(&state.db, room.id).await?;
    Ok(ApiResponse::ok(room))
}

#[derive(Deserialize)]
struct SetPlayingReq {
    file_id: Uuid,
}

async fn set_playing(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    axum::Json(body): axum::Json<SetPlayingReq>,
) -> Result<ApiResponse<()>, AppError> {
    if !RoomService::is_host(&state.db, id, auth.user_id).await? {
        return Err(AppError::Forbidden);
    }

    let room =
        RoomService::update_playback(&state.db, id, Some(body.file_id), None, Some("waiting"))
            .await?;

    let tx = get_or_create_channel(&state.room_channels, id);
    let msg = WsOut::Sync {
        status: room.status,
        time: room.current_time,
        file_id: room.current_file_id.map(|fid: Uuid| fid.to_string()),
        server_time: now_secs(),
    };
    let _ = tx.send(serde_json::to_string(&msg).unwrap_or_default());

    Ok(ApiResponse::ok(()))
}

async fn list_members(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<Vec<MemberInfo>>, AppError> {
    if !RoomService::is_member(&state.db, id, auth.user_id).await? {
        return Err(AppError::Forbidden);
    }
    let members = RoomService::list_members(&state.db, id).await?;
    Ok(ApiResponse::ok(members))
}

// --- WebSocket handler ---

#[derive(Deserialize)]
struct WsQuery {
    #[allow(dead_code)]
    token: Option<String>,
}

async fn ws_upgrade(
    State(state): State<AppState>,
    Path(room_id): Path<Uuid>,
    Query(_query): Query<WsQuery>,
    auth: AuthUser,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, AppError> {
    if !RoomService::is_member(&state.db, room_id, auth.user_id).await? {
        return Err(AppError::Forbidden);
    }

    let user: mdp_core::entity::users::Model =
        mdp_core::entity::users::Entity::find_by_id(auth.user_id)
            .one(&state.db)
            .await
            .map_err(|e: sea_orm::DbErr| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound("用户不存在".to_string()))?;

    let user_brief = UserBrief {
        id: user.id.to_string(),
        name: user.username.clone(),
        avatar: user.avatar.clone(),
    };

    Ok(ws.on_upgrade(move |socket| {
        handle_ws(socket, state, room_id, auth.user_id, user_brief)
    }))
}

async fn handle_ws(
    socket: WebSocket,
    state: AppState,
    room_id: Uuid,
    user_id: Uuid,
    user_brief: UserBrief,
) {
    let (mut ws_tx, mut ws_rx) = socket.split();

    let tx = get_or_create_channel(&state.room_channels, room_id);
    let mut rx = tx.subscribe();

    // Broadcast member_join
    let join_msg = WsOut::MemberJoin {
        user: user_brief.clone(),
    };
    let _ = tx.send(serde_json::to_string(&join_msg).unwrap_or_default());

    // Send initial sync
    if let Ok(Some(room)) = mdp_core::entity::rooms::Entity::find_by_id(room_id)
        .one(&state.db)
        .await
    {
        let sync = WsOut::Sync {
            status: room.status.clone(),
            time: room.current_time,
            file_id: room.current_file_id.map(|fid: Uuid| fid.to_string()),
            server_time: now_secs(),
        };
        let json = serde_json::to_string(&sync).unwrap_or_default();
        let _ = ws_tx.send(Message::Text(json.into())).await;
    }

    let user_id_str = user_id.to_string();
    let tx2 = tx.clone();
    let db = state.db.clone();
    let channels = state.room_channels.clone();

    // Read loop: client → server
    let user_brief_clone = user_brief.clone();
    let read_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            let text = match msg {
                Message::Text(t) => t,
                Message::Close(_) => break,
                _ => continue,
            };

            let Ok(ws_in) = serde_json::from_str::<WsIn>(&text) else {
                continue;
            };

            match ws_in {
                WsIn::Ping {} => {
                    let pong = WsOut::Pong {
                        server_time: now_secs(),
                    };
                    let payload = serde_json::to_string(&pong).unwrap_or_default();
                    let _ = tx2.send(format!("__direct__{user_id}|{payload}"));
                }
                WsIn::Play {} => {
                    if let Ok(true) = RoomService::is_host(&db, room_id, user_id).await {
                        if let Ok(room) = RoomService::update_playback(
                            &db, room_id, None, None, Some("playing"),
                        )
                        .await
                        {
                            broadcast_sync(&tx2, &room);
                        }
                    }
                }
                WsIn::Pause {} => {
                    if let Ok(true) = RoomService::is_host(&db, room_id, user_id).await {
                        if let Ok(room) = RoomService::update_playback(
                            &db, room_id, None, None, Some("paused"),
                        )
                        .await
                        {
                            broadcast_sync(&tx2, &room);
                        }
                    }
                }
                WsIn::Seek { time } => {
                    if let Ok(true) = RoomService::is_host(&db, room_id, user_id).await {
                        if let Ok(room) = RoomService::update_playback(
                            &db, room_id, None, Some(time), None,
                        )
                        .await
                        {
                            broadcast_sync(&tx2, &room);
                        }
                    }
                }
                WsIn::Chat { content } => {
                    let chat = WsOut::Chat {
                        user: user_brief_clone.clone(),
                        content,
                    };
                    let _ = tx2.send(serde_json::to_string(&chat).unwrap_or_default());
                }
                WsIn::Danmaku {
                    content,
                    color,
                    position,
                } => {
                    let danmaku = WsOut::Danmaku {
                        user_id: user_id.to_string(),
                        content,
                        color,
                        position,
                        video_time: 0.0,
                    };
                    let _ = tx2.send(serde_json::to_string(&danmaku).unwrap_or_default());
                }
            }
        }
    });

    // Write loop: server → client
    let write_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    if let Some(rest) = msg.strip_prefix("__direct__") {
                        if let Some((target_id, payload)) = rest.split_once('|') {
                            if target_id == user_id_str {
                                if ws_tx
                                    .send(Message::Text(payload.to_string().into()))
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                        }
                        continue;
                    }
                    if ws_tx.send(Message::Text(msg.into())).await.is_err() {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(_) => break,
            }
        }
    });

    tokio::select! {
        _ = read_task => {},
        _ = write_task => {},
    }

    // Cleanup
    let leave_msg = WsOut::MemberLeave {
        user_id: user_id.to_string(),
    };
    let _ = tx.send(serde_json::to_string(&leave_msg).unwrap_or_default());
    cleanup_channel(&channels, room_id);
}

fn broadcast_sync(
    tx: &tokio::sync::broadcast::Sender<String>,
    room: &mdp_core::entity::rooms::Model,
) {
    let sync = WsOut::Sync {
        status: room.status.clone(),
        time: room.current_time,
        file_id: room.current_file_id.map(|fid: Uuid| fid.to_string()),
        server_time: now_secs(),
    };
    let _ = tx.send(serde_json::to_string(&sync).unwrap_or_default());
}
