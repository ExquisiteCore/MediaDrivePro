use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

/// Shared map of room_id → broadcast sender.
/// Each room gets a broadcast channel; all connected WS clients subscribe.
pub type RoomChannels = Arc<DashMap<Uuid, broadcast::Sender<String>>>;

/// Get or create a broadcast channel for a room (capacity 256 messages).
pub fn get_or_create_channel(channels: &RoomChannels, room_id: Uuid) -> broadcast::Sender<String> {
    channels
        .entry(room_id)
        .or_insert_with(|| broadcast::channel(256).0)
        .clone()
}

/// Remove channel if no subscribers remain.
pub fn cleanup_channel(channels: &RoomChannels, room_id: Uuid) {
    if let Some(entry) = channels.get(&room_id) {
        if entry.receiver_count() == 0 {
            drop(entry);
            channels.remove(&room_id);
        }
    }
}

// --- Server → Client messages ---

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum WsOut {
    #[serde(rename = "sync")]
    Sync {
        status: String,
        time: f64,
        file_id: Option<String>,
        server_time: f64,
    },
    #[serde(rename = "member_join")]
    MemberJoin { user: UserBrief },
    #[serde(rename = "member_leave")]
    MemberLeave { user_id: String },
    #[serde(rename = "chat")]
    Chat { user: UserBrief, content: String },
    #[serde(rename = "danmaku")]
    Danmaku {
        user_id: String,
        content: String,
        color: String,
        position: String,
        video_time: f64,
    },
    #[serde(rename = "pong")]
    Pong { server_time: f64 },
    #[serde(rename = "error")]
    Error { code: String, message: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserBrief {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
}

// --- Client → Server messages ---

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum WsIn {
    #[serde(rename = "play")]
    Play {},
    #[serde(rename = "pause")]
    Pause {},
    #[serde(rename = "seek")]
    Seek { time: f64 },
    #[serde(rename = "chat")]
    Chat { content: String },
    #[serde(rename = "danmaku")]
    Danmaku {
        content: String,
        #[serde(default = "default_color")]
        color: String,
        #[serde(default = "default_position")]
        position: String,
    },
    #[serde(rename = "ping")]
    Ping {},
}

fn default_color() -> String {
    "#FFFFFF".to_string()
}
fn default_position() -> String {
    "scroll".to_string()
}

/// Helper: get current unix timestamp as f64.
pub fn now_secs() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}
