use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// In-memory room state (inspired by synctv's Current + Status model)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayStatus {
    Waiting,
    Playing,
    Paused,
}

impl PlayStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PlayStatus::Waiting => "waiting",
            PlayStatus::Playing => "playing",
            PlayStatus::Paused => "paused",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "playing" => PlayStatus::Playing,
            "paused" => PlayStatus::Paused,
            _ => PlayStatus::Waiting,
        }
    }
}

/// Authoritative playback state held in memory.
/// The server continuously tracks elapsed time so any client can ask
/// "where are we right now?" and get an accurate answer.
pub struct RoomState {
    pub status: PlayStatus,
    /// Playback position (seconds) at `last_update`.
    pub current_time: f64,
    pub playback_rate: f64,
    pub last_update: Instant,
    pub file_id: Option<Uuid>,
}

impl RoomState {
    pub fn new() -> Self {
        Self {
            status: PlayStatus::Waiting,
            current_time: 0.0,
            playback_rate: 1.0,
            last_update: Instant::now(),
            file_id: None,
        }
    }

    /// Restore state from database values on first connect.
    pub fn from_db(status: &str, current_time: f64, file_id: Option<Uuid>) -> Self {
        Self {
            status: PlayStatus::from_str(status),
            current_time,
            playback_rate: 1.0,
            last_update: Instant::now(),
            file_id,
        }
    }

    /// Compute the real-time playback position by adding elapsed time.
    /// Mirrors synctv's `Current.UpdateStatus()`.
    pub fn current_position(&self) -> f64 {
        if self.status == PlayStatus::Playing {
            self.current_time + self.last_update.elapsed().as_secs_f64() * self.playback_rate
        } else {
            self.current_time
        }
    }

    /// Snapshot the current position and reset `last_update`.
    pub fn update(&mut self) {
        self.current_time = self.current_position();
        self.last_update = Instant::now();
    }

    /// Set full playback status with network-delay compensation.
    /// `time_diff` is the estimated seconds between client-send and server-receive.
    pub fn set_status(&mut self, playing: bool, seek: f64, rate: f64, time_diff: f64) {
        self.update(); // snapshot before changing
        self.status = if playing {
            PlayStatus::Playing
        } else {
            PlayStatus::Paused
        };
        self.playback_rate = rate;
        self.current_time = if playing {
            seek + time_diff * rate
        } else {
            seek
        };
        self.last_update = Instant::now();
    }

    /// Seek to a position.
    pub fn seek(&mut self, time: f64, time_diff: f64) {
        self.update();
        self.current_time = if self.status == PlayStatus::Playing {
            time + time_diff * self.playback_rate
        } else {
            time
        };
        self.last_update = Instant::now();
    }

    /// Set playing / paused without changing position.
    pub fn set_playing(&mut self, playing: bool, time_diff: f64) {
        self.update();
        self.status = if playing {
            PlayStatus::Playing
        } else {
            PlayStatus::Paused
        };
        // When resuming, compensate for delay
        if playing {
            self.current_time += time_diff * self.playback_rate;
        }
        self.last_update = Instant::now();
    }

    /// Set a new file, reset position.
    pub fn set_file(&mut self, file_id: Uuid) {
        self.file_id = Some(file_id);
        self.current_time = 0.0;
        self.status = PlayStatus::Waiting;
        self.playback_rate = 1.0;
        self.last_update = Instant::now();
    }
}

// ---------------------------------------------------------------------------
// Room Hub: broadcast channel + shared state
// ---------------------------------------------------------------------------

pub struct RoomHub {
    pub tx: broadcast::Sender<String>,
    pub state: Arc<RwLock<RoomState>>,
}

impl RoomHub {
    pub fn new(initial_state: RoomState) -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            tx,
            state: Arc::new(RwLock::new(initial_state)),
        }
    }
}

/// Shared map of room_id → RoomHub.
pub type RoomChannels = Arc<DashMap<Uuid, RoomHub>>;

/// Get or create a hub for a room. When creating, initialize from DB values.
pub fn get_or_create_hub(
    channels: &RoomChannels,
    room_id: Uuid,
    initial_state: RoomState,
) -> (broadcast::Sender<String>, Arc<RwLock<RoomState>>) {
    let hub = channels
        .entry(room_id)
        .or_insert_with(|| RoomHub::new(initial_state));
    (hub.tx.clone(), hub.state.clone())
}

/// Remove hub if no subscribers remain.
pub fn cleanup_hub(channels: &RoomChannels, room_id: Uuid) {
    if let Some(hub) = channels.get(&room_id) {
        if hub.tx.receiver_count() == 0 {
            drop(hub);
            channels.remove(&room_id);
        }
    }
}

// ---------------------------------------------------------------------------
// WS message types
// ---------------------------------------------------------------------------

// --- Server → Client ---

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum WsOut {
    #[serde(rename = "sync")]
    Sync {
        status: String,
        time: f64,
        playback_rate: f64,
        file_id: Option<String>,
        server_time: f64,
    },
    #[serde(rename = "check_status")]
    CheckStatus {
        is_playing: bool,
        current_time: f64,
        playback_rate: f64,
    },
    #[serde(rename = "viewer_count")]
    ViewerCount { count: usize },
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

// --- Client → Server ---

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum WsIn {
    #[serde(rename = "play")]
    Play {
        #[serde(default)]
        timestamp: i64,
    },
    #[serde(rename = "pause")]
    Pause {
        #[serde(default)]
        timestamp: i64,
    },
    #[serde(rename = "seek")]
    Seek {
        time: f64,
        #[serde(default)]
        timestamp: i64,
    },
    #[serde(rename = "check_status")]
    CheckStatus {
        is_playing: bool,
        current_time: f64,
        playback_rate: f64,
        #[serde(default)]
        timestamp: i64,
    },
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Current unix timestamp in seconds (f64).
pub fn now_secs() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

/// Current unix timestamp in milliseconds.
pub fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// Calculate network delay from client timestamp.
/// Clamped to [0, 1.5] seconds (same as synctv).
pub fn calculate_time_diff(client_timestamp_ms: i64) -> f64 {
    if client_timestamp_ms == 0 {
        return 0.0;
    }
    let diff = (now_millis() - client_timestamp_ms) as f64 / 1000.0;
    diff.clamp(0.0, 1.5)
}

/// Max allowed drift (seconds) before server sends correction.
pub const MAX_DRIFT: f64 = 10.0;

/// Max chat message length.
pub const MAX_CHAT_LENGTH: usize = 4096;
