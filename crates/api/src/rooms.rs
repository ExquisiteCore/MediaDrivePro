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
use std::time::Duration;
use tokio::sync::mpsc;
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

// ===========================================================================
// REST handlers
// ===========================================================================

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

    if let Some(hub) = state.room_channels.get(&id) {
        let msg = WsOut::Error {
            code: "ROOM_CLOSED".to_string(),
            message: "房间已关闭".to_string(),
        };
        let _ = hub.tx.send(serde_json::to_string(&msg).unwrap_or_default());
    }
    cleanup_hub(&state.room_channels, id);

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

    // Persist to DB
    let _room =
        RoomService::update_playback(&state.db, id, Some(body.file_id), None, Some("waiting"))
            .await?;

    // Update in-memory state + broadcast
    if let Some(hub) = state.room_channels.get(&id) {
        let mut st = hub.state.write().await;
        st.set_file(body.file_id);
        broadcast_sync_from_state(&hub.tx, &st);
    }

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

// ===========================================================================
// WebSocket handler
// ===========================================================================

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

    // Load or create in-memory hub with state from DB
    let initial_state = match mdp_core::entity::rooms::Entity::find_by_id(room_id)
        .one(&state.db)
        .await
    {
        Ok(Some(room)) => RoomState::from_db(
            &room.status,
            room.current_time,
            room.current_file_id,
        ),
        _ => RoomState::new(),
    };

    let (tx, hub_state) = get_or_create_hub(&state.room_channels, room_id, initial_state);
    let mut rx = tx.subscribe();

    // Broadcast member_join
    let join_msg = WsOut::MemberJoin {
        user: user_brief.clone(),
    };
    let _ = tx.send(serde_json::to_string(&join_msg).unwrap_or_default());

    // Send initial sync to this client
    {
        let st = hub_state.read().await;
        let sync = make_sync_msg(&st);
        let json = serde_json::to_string(&sync).unwrap_or_default();
        let _ = ws_tx.send(Message::Text(json.into())).await;

        // Send initial viewer count
        let vc = WsOut::ViewerCount {
            count: tx.receiver_count(),
        };
        let json = serde_json::to_string(&vc).unwrap_or_default();
        let _ = ws_tx.send(Message::Text(json.into())).await;
    }

    // Unicast channel (for pong + check_status responses)
    let (direct_tx, mut direct_rx) = mpsc::channel::<String>(32);

    let user_id_str = user_id.to_string();
    let tx_read = tx.clone();
    let hub_state_read = hub_state.clone();
    let db_read = state.db.clone();

    // -----------------------------------------------------------------------
    // Read loop: client → server
    // -----------------------------------------------------------------------
    let user_brief_clone = user_brief.clone();
    let direct_tx_read = direct_tx.clone();
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
                    let _ = direct_tx_read
                        .send(serde_json::to_string(&pong).unwrap_or_default())
                        .await;
                }

                WsIn::Play { timestamp } => {
                    if let Ok(true) = RoomService::is_host(&db_read, room_id, user_id).await {
                        let time_diff = calculate_time_diff(timestamp);
                        let mut st = hub_state_read.write().await;
                        st.set_playing(true, time_diff);
                        broadcast_sync_from_state(&tx_read, &st);
                    }
                }

                WsIn::Pause { timestamp } => {
                    if let Ok(true) = RoomService::is_host(&db_read, room_id, user_id).await {
                        let time_diff = calculate_time_diff(timestamp);
                        let mut st = hub_state_read.write().await;
                        st.set_playing(false, time_diff);
                        broadcast_sync_from_state(&tx_read, &st);
                    }
                }

                WsIn::Seek { time, timestamp } => {
                    if let Ok(true) = RoomService::is_host(&db_read, room_id, user_id).await {
                        let time_diff = calculate_time_diff(timestamp);
                        let mut st = hub_state_read.write().await;
                        st.seek(time, time_diff);
                        broadcast_sync_from_state(&tx_read, &st);
                    }
                }

                WsIn::CheckStatus {
                    is_playing,
                    current_time,
                    playback_rate,
                    timestamp,
                } => {
                    let time_diff = calculate_time_diff(timestamp);
                    let st = hub_state_read.read().await;
                    let server_pos = st.current_position();
                    let client_pos = current_time + time_diff;

                    let needs_correction =
                        is_playing != (st.status == PlayStatus::Playing)
                            || playback_rate != st.playback_rate
                            || (server_pos - client_pos).abs() > MAX_DRIFT;

                    if needs_correction {
                        let correction = WsOut::CheckStatus {
                            is_playing: st.status == PlayStatus::Playing,
                            current_time: st.current_position(),
                            playback_rate: st.playback_rate,
                        };
                        let _ = direct_tx_read
                            .send(serde_json::to_string(&correction).unwrap_or_default())
                            .await;
                    }
                }

                WsIn::Chat { content } => {
                    let content = content.trim().to_string();
                    if content.is_empty() || content.len() > MAX_CHAT_LENGTH {
                        continue;
                    }
                    let safe = html_escape::encode_text(&content).to_string();
                    let chat = WsOut::Chat {
                        user: user_brief_clone.clone(),
                        content: safe,
                    };
                    let _ = tx_read.send(serde_json::to_string(&chat).unwrap_or_default());
                }

                WsIn::Danmaku {
                    content,
                    color,
                    position,
                } => {
                    let content = content.trim().to_string();
                    if content.is_empty() || content.len() > 200 {
                        continue;
                    }
                    let safe = html_escape::encode_text(&content).to_string();
                    let danmaku = WsOut::Danmaku {
                        user_id: user_id.to_string(),
                        content: safe,
                        color,
                        position,
                        video_time: {
                            let st = hub_state_read.read().await;
                            st.current_position()
                        },
                    };
                    let _ = tx_read.send(serde_json::to_string(&danmaku).unwrap_or_default());
                }
            }
        }
    });

    // -----------------------------------------------------------------------
    // Write loop: server → client (broadcast + unicast)
    // -----------------------------------------------------------------------
    let write_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Broadcast messages
                result = rx.recv() => {
                    match result {
                        Ok(msg) => {
                            if ws_tx.send(Message::Text(msg.into())).await.is_err() {
                                break;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(_) => break,
                    }
                }
                // Unicast messages (pong, check_status corrections)
                result = direct_rx.recv() => {
                    match result {
                        Some(msg) => {
                            if ws_tx.send(Message::Text(msg.into())).await.is_err() {
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    });

    // -----------------------------------------------------------------------
    // Heartbeat: periodic sync broadcast + viewer count
    // -----------------------------------------------------------------------
    let tx_hb = tx.clone();
    let hub_state_hb = hub_state.clone();
    let heartbeat_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        let mut last_count: usize = 0;
        loop {
            interval.tick().await;

            // Viewer count
            let count = tx_hb.receiver_count();
            if count != last_count {
                let vc = WsOut::ViewerCount { count };
                let _ = tx_hb.send(serde_json::to_string(&vc).unwrap_or_default());
                last_count = count;
            }

            // Periodic sync when playing
            let st = hub_state_hb.read().await;
            if st.status == PlayStatus::Playing {
                broadcast_sync_from_state(&tx_hb, &st);
            }
        }
    });

    // Wait for any task to finish
    tokio::select! {
        _ = read_task => {},
        _ = write_task => {},
        _ = heartbeat_task => {},
    }

    // -----------------------------------------------------------------------
    // Cleanup on disconnect
    // -----------------------------------------------------------------------
    let leave_msg = WsOut::MemberLeave {
        user_id: user_id_str,
    };
    let _ = tx.send(serde_json::to_string(&leave_msg).unwrap_or_default());

    // If last subscriber, persist state to DB then remove hub
    if tx.receiver_count() <= 1 {
        let st = hub_state.read().await;
        let _ = RoomService::update_playback(
            &state.db,
            room_id,
            st.file_id,
            Some(st.current_position()),
            Some(st.status.as_str()),
        )
        .await;
        state.room_channels.remove(&room_id);
    }
}

// ===========================================================================
// Helpers
// ===========================================================================

fn make_sync_msg(state: &RoomState) -> WsOut {
    WsOut::Sync {
        status: state.status.as_str().to_string(),
        time: state.current_position(),
        playback_rate: state.playback_rate,
        file_id: state.file_id.map(|fid| fid.to_string()),
        server_time: now_secs(),
    }
}

fn broadcast_sync_from_state(tx: &tokio::sync::broadcast::Sender<String>, state: &RoomState) {
    let sync = make_sync_msg(state);
    let _ = tx.send(serde_json::to_string(&sync).unwrap_or_default());
}
