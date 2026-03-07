use chrono::Utc;
use mdp_common::{config::VideoConfig, error::AppError};
use mdp_storage::operator::storage_key;
use opendal::Operator;
use sea_orm::*;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use uuid::Uuid;

use crate::entity::{files, transcode_tasks};

#[derive(Debug, serde::Serialize)]
pub struct TranscodeTaskInfo {
    pub id: Uuid,
    pub file_id: Uuid,
    pub status: String,
    pub profile: String,
    pub progress: i16,
    pub output_key: Option<String>,
    pub error_msg: Option<String>,
    pub started_at: Option<chrono::DateTime<Utc>>,
    pub completed_at: Option<chrono::DateTime<Utc>>,
    pub created_at: chrono::DateTime<Utc>,
}

impl From<transcode_tasks::Model> for TranscodeTaskInfo {
    fn from(t: transcode_tasks::Model) -> Self {
        Self {
            id: t.id,
            file_id: t.file_id,
            status: t.status,
            profile: t.profile,
            progress: t.progress,
            output_key: t.output_key,
            error_msg: t.error_msg,
            started_at: t.started_at,
            completed_at: t.completed_at,
            created_at: t.created_at,
        }
    }
}

/// Profile parameters for FFmpeg transcoding.
pub struct TranscodeProfile {
    pub name: &'static str,
    pub video_bitrate: &'static str,
    pub audio_bitrate: &'static str,
    pub scale_filter: &'static str,
    pub resolution: &'static str,
    pub bandwidth: u64,
}

const PROFILES: &[TranscodeProfile] = &[
    TranscodeProfile {
        name: "480p",
        video_bitrate: "1000k",
        audio_bitrate: "128k",
        scale_filter: "scale=-2:480",
        resolution: "854x480",
        bandwidth: 1_000_000,
    },
    TranscodeProfile {
        name: "720p",
        video_bitrate: "2500k",
        audio_bitrate: "128k",
        scale_filter: "scale=-2:720",
        resolution: "1280x720",
        bandwidth: 2_500_000,
    },
    TranscodeProfile {
        name: "1080p",
        video_bitrate: "5000k",
        audio_bitrate: "192k",
        scale_filter: "scale=-2:1080",
        resolution: "1920x1080",
        bandwidth: 5_000_000,
    },
];

fn get_profile(name: &str) -> Option<&'static TranscodeProfile> {
    PROFILES.iter().find(|p| p.name == name)
}

const VIDEO_CONTENT_TYPES: &[&str] = &[
    "video/mp4",
    "video/webm",
    "video/ogg",
    "video/x-matroska",
    "video/x-msvideo",
    "video/quicktime",
    "video/x-flv",
    "video/mpeg",
];

pub struct TranscodeService;

impl TranscodeService {
    /// Create a new transcode task for a file.
    pub async fn create(
        db: &DatabaseConnection,
        user_id: Uuid,
        file_id: Uuid,
        profile: &str,
    ) -> Result<TranscodeTaskInfo, AppError> {
        // Validate profile
        if get_profile(profile).is_none() {
            return Err(AppError::Validation(format!(
                "不支持的转码配置: {profile}，可选: 480p, 720p, 1080p"
            )));
        }

        // Verify file exists and is a video
        let file = files::Entity::find_by_id(file_id)
            .filter(files::Column::UserId.eq(user_id))
            .filter(files::Column::Status.ne("deleted"))
            .one(db)
            .await?
            .ok_or(AppError::NotFound("文件不存在".to_string()))?;

        if !VIDEO_CONTENT_TYPES.contains(&file.content_type.as_str()) {
            return Err(AppError::Validation("该文件不是视频文件".to_string()));
        }

        // Check if same profile task already exists (not failed)
        let existing = transcode_tasks::Entity::find()
            .filter(transcode_tasks::Column::FileId.eq(file_id))
            .filter(transcode_tasks::Column::Profile.eq(profile))
            .filter(transcode_tasks::Column::Status.ne("failed"))
            .one(db)
            .await?;

        if let Some(existing) = existing {
            return Ok(existing.into());
        }

        let task = transcode_tasks::ActiveModel {
            id: Set(Uuid::new_v4()),
            file_id: Set(file_id),
            status: Set("pending".to_string()),
            profile: Set(profile.to_string()),
            progress: Set(0),
            output_key: Set(None),
            error_msg: Set(None),
            retry_count: Set(0),
            started_at: Set(None),
            completed_at: Set(None),
            created_at: Set(Utc::now()),
        };

        let model = task.insert(db).await?;
        Ok(model.into())
    }

    pub async fn get(
        db: &DatabaseConnection,
        task_id: Uuid,
    ) -> Result<TranscodeTaskInfo, AppError> {
        let task = transcode_tasks::Entity::find_by_id(task_id)
            .one(db)
            .await?
            .ok_or(AppError::NotFound("转码任务不存在".to_string()))?;
        Ok(task.into())
    }

    pub async fn list_by_file(
        db: &DatabaseConnection,
        file_id: Uuid,
    ) -> Result<Vec<TranscodeTaskInfo>, AppError> {
        let tasks = transcode_tasks::Entity::find()
            .filter(transcode_tasks::Column::FileId.eq(file_id))
            .order_by_desc(transcode_tasks::Column::CreatedAt)
            .all(db)
            .await?
            .into_iter()
            .map(TranscodeTaskInfo::from)
            .collect();
        Ok(tasks)
    }

    /// Poll for the next pending task and mark it as processing.
    pub async fn poll_pending(
        db: &DatabaseConnection,
    ) -> Result<Option<transcode_tasks::Model>, AppError> {
        let task = transcode_tasks::Entity::find()
            .filter(transcode_tasks::Column::Status.eq("pending"))
            .filter(transcode_tasks::Column::RetryCount.lt(3i16))
            .order_by_asc(transcode_tasks::Column::CreatedAt)
            .one(db)
            .await?;

        if let Some(task) = task {
            let mut active: transcode_tasks::ActiveModel = task.clone().into();
            active.status = Set("processing".to_string());
            active.started_at = Set(Some(Utc::now()));
            active.update(db).await?;
            Ok(Some(task))
        } else {
            Ok(None)
        }
    }

    pub async fn mark_completed(
        db: &DatabaseConnection,
        task_id: Uuid,
        output_key: &str,
    ) -> Result<(), AppError> {
        transcode_tasks::Entity::update_many()
            .col_expr(
                transcode_tasks::Column::Status,
                sea_orm::sea_query::Expr::value("completed"),
            )
            .col_expr(
                transcode_tasks::Column::OutputKey,
                sea_orm::sea_query::Expr::value(output_key),
            )
            .col_expr(
                transcode_tasks::Column::Progress,
                sea_orm::sea_query::Expr::value(100i16),
            )
            .col_expr(
                transcode_tasks::Column::CompletedAt,
                sea_orm::sea_query::Expr::value(Utc::now()),
            )
            .filter(transcode_tasks::Column::Id.eq(task_id))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn mark_failed(
        db: &DatabaseConnection,
        task_id: Uuid,
        error_msg: &str,
    ) -> Result<(), AppError> {
        let task = transcode_tasks::Entity::find_by_id(task_id)
            .one(db)
            .await?
            .ok_or(AppError::NotFound("任务不存在".to_string()))?;

        let new_status = if task.retry_count >= 2 {
            "failed"
        } else {
            "pending"
        };

        let mut active: transcode_tasks::ActiveModel = task.into();
        active.status = Set(new_status.to_string());
        active.error_msg = Set(Some(error_msg.to_string()));
        active.retry_count = Set(active.retry_count.unwrap() + 1);
        active.update(db).await?;
        Ok(())
    }

    pub async fn update_progress(
        db: &DatabaseConnection,
        task_id: Uuid,
        progress: i16,
    ) -> Result<(), AppError> {
        transcode_tasks::Entity::update_many()
            .col_expr(
                transcode_tasks::Column::Progress,
                sea_orm::sea_query::Expr::value(progress),
            )
            .filter(transcode_tasks::Column::Id.eq(task_id))
            .exec(db)
            .await?;
        Ok(())
    }
}

/// Run FFmpeg transcoding for a task.
pub async fn run_transcode(
    db: &DatabaseConnection,
    storage: &Operator,
    config: &VideoConfig,
    task: transcode_tasks::Model,
) {
    let task_id = task.id;
    let file_id = task.file_id;
    let profile_name = &task.profile;

    let result = run_transcode_inner(db, storage, config, file_id, profile_name).await;

    match result {
        Ok(output_key) => {
            if let Err(e) = TranscodeService::mark_completed(db, task_id, &output_key).await {
                tracing::error!("Failed to mark task {task_id} as completed: {e}");
            }
            tracing::info!("Transcode task {task_id} completed: {output_key}");
        }
        Err(e) => {
            let msg = format!("{e}");
            tracing::error!("Transcode task {task_id} failed: {msg}");
            if let Err(e2) = TranscodeService::mark_failed(db, task_id, &msg).await {
                tracing::error!("Failed to mark task {task_id} as failed: {e2}");
            }
        }
    }
}

async fn run_transcode_inner(
    db: &DatabaseConnection,
    storage: &Operator,
    config: &VideoConfig,
    file_id: Uuid,
    profile_name: &str,
) -> Result<String, AppError> {
    let profile = get_profile(profile_name)
        .ok_or_else(|| AppError::Validation(format!("Unknown profile: {profile_name}")))?;

    // Get file info
    let file = files::Entity::find_by_id(file_id)
        .one(db)
        .await?
        .ok_or(AppError::NotFound("文件不存在".to_string()))?;

    // Create temp directory
    let tmp_dir = std::env::temp_dir().join(format!("mdp_transcode_{}", Uuid::new_v4()));
    tokio::fs::create_dir_all(&tmp_dir)
        .await
        .map_err(|e| AppError::Internal(format!("创建临时目录失败: {e}")))?;

    let cleanup = TempDirGuard(tmp_dir.clone());

    // Download source file to temp dir
    let ext = file
        .name
        .rsplit('.')
        .next()
        .unwrap_or("mp4");
    let input_path = tmp_dir.join(format!("input.{ext}"));

    let data = storage
        .read(&file.storage_key)
        .await
        .map_err(|e| AppError::Internal(format!("下载源文件失败: {e}")))?
        .to_vec();

    tokio::fs::write(&input_path, &data)
        .await
        .map_err(|e| AppError::Internal(format!("写入临时文件失败: {e}")))?;

    // Create output directory
    let output_dir = tmp_dir.join(profile_name);
    tokio::fs::create_dir_all(&output_dir)
        .await
        .map_err(|e| AppError::Internal(format!("创建输出目录失败: {e}")))?;

    // Run FFmpeg
    let segment_pattern = output_dir
        .join("segment_%04d.ts")
        .to_string_lossy()
        .to_string();
    let output_m3u8 = output_dir
        .join("index.m3u8")
        .to_string_lossy()
        .to_string();

    let status = Command::new(&config.ffmpeg_path)
        .args([
            "-i",
            input_path.to_str().unwrap(),
            "-c:v",
            "libx264",
            "-preset",
            "medium",
            "-b:v",
            profile.video_bitrate,
            "-c:a",
            "aac",
            "-b:a",
            profile.audio_bitrate,
            "-vf",
            profile.scale_filter,
            "-f",
            "hls",
            "-hls_time",
            "6",
            "-hls_list_size",
            "0",
            "-hls_segment_filename",
            &segment_pattern,
            &output_m3u8,
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .status()
        .await
        .map_err(|e| AppError::Internal(format!("FFmpeg 启动失败: {e}")))?;

    if !status.success() {
        return Err(AppError::Internal(format!(
            "FFmpeg 退出码: {}",
            status.code().unwrap_or(-1)
        )));
    }

    // Upload output files to storage
    let mut entries = tokio::fs::read_dir(&output_dir)
        .await
        .map_err(|e| AppError::Internal(format!("读取输出目录失败: {e}")))?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| AppError::Internal(format!("读取目录条目失败: {e}")))?
    {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let file_data = tokio::fs::read(entry.path())
            .await
            .map_err(|e| AppError::Internal(format!("读取输出文件失败: {e}")))?;

        let storage_path = format!(
            "{}/{profile_name}/{file_name}",
            storage_key::transcode_dir(file_id)
        );
        storage
            .write(&storage_path, file_data)
            .await
            .map_err(|e| AppError::Internal(format!("上传转码文件失败: {e}")))?;
    }

    // Generate master m3u8
    let master_content = format!(
        "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH={},RESOLUTION={}\n{profile_name}/index.m3u8\n",
        profile.bandwidth, profile.resolution
    );

    let master_key = storage_key::transcode_master(file_id);
    storage
        .write(&master_key, master_content.into_bytes())
        .await
        .map_err(|e| AppError::Internal(format!("上传 master m3u8 失败: {e}")))?;

    // Try to get duration/resolution with ffprobe and update media_info
    let _ = update_media_from_ffprobe(db, config, &input_path, file_id, profile_name).await;

    // Look for subtitle files (same directory as the source file — check by name prefix)
    let _ = extract_subtitles(storage, &tmp_dir, &input_path, config, file_id).await;

    drop(cleanup);

    Ok(master_key)
}

/// Use ffprobe to get duration and resolution, update media_info if it exists.
async fn update_media_from_ffprobe(
    db: &DatabaseConnection,
    config: &VideoConfig,
    input_path: &Path,
    file_id: Uuid,
    _profile: &str,
) -> Result<(), AppError> {
    let output = Command::new(&config.ffprobe_path)
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            input_path.to_str().unwrap(),
        ])
        .output()
        .await
        .map_err(|e| AppError::Internal(format!("ffprobe 失败: {e}")))?;

    if !output.status.success() {
        return Ok(());
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_default();

    // Extract duration
    let duration = json["format"]["duration"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .map(|d| d as i32);

    // Extract resolution from video stream
    let resolution = json["streams"]
        .as_array()
        .and_then(|streams| {
            streams.iter().find(|s| s["codec_type"] == "video")
        })
        .and_then(|vs| {
            let w = vs["width"].as_i64()?;
            let h = vs["height"].as_i64()?;
            let res = if w >= 3840 || h >= 2160 {
                "4K"
            } else if h >= 1080 {
                "1080p"
            } else if h >= 720 {
                "720p"
            } else if h >= 480 {
                "480p"
            } else {
                "SD"
            };
            Some(res.to_string())
        });

    // Update or create media_info
    use crate::entity::media_info;

    let existing = media_info::Entity::find()
        .filter(media_info::Column::FileId.eq(file_id))
        .one(db)
        .await?;

    if let Some(existing) = existing {
        let mut active: media_info::ActiveModel = existing.into();
        if let Some(d) = duration {
            active.duration = Set(Some(d));
        }
        if let Some(ref r) = resolution {
            active.resolution = Set(Some(r.clone()));
        }
        active.update(db).await?;
    } else {
        let info = media_info::ActiveModel {
            id: Set(Uuid::new_v4()),
            file_id: Set(file_id),
            media_type: Set("movie".to_string()),
            title: Set(None),
            season: Set(None),
            episode: Set(None),
            tmdb_id: Set(None),
            poster_url: Set(None),
            overview: Set(None),
            year: Set(None),
            duration: Set(duration),
            resolution: Set(resolution),
            created_at: Set(Utc::now()),
        };
        info.insert(db).await?;
    }

    Ok(())
}

/// Try to extract embedded subtitles using FFmpeg.
async fn extract_subtitles(
    storage: &Operator,
    tmp_dir: &Path,
    input_path: &Path,
    config: &VideoConfig,
    file_id: Uuid,
) -> Result<(), AppError> {
    let subs_dir = tmp_dir.join("subtitles");
    tokio::fs::create_dir_all(&subs_dir).await.ok();

    let vtt_path = subs_dir.join("default.vtt");

    // Try to extract first subtitle stream as VTT
    let status = Command::new(&config.ffmpeg_path)
        .args([
            "-i",
            input_path.to_str().unwrap(),
            "-map",
            "0:s:0",
            "-c:s",
            "webvtt",
            vtt_path.to_str().unwrap(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await;

    if let Ok(s) = status {
        if s.success() {
            if let Ok(data) = tokio::fs::read(&vtt_path).await {
                let key = storage_key::subtitle(file_id, "default.vtt");
                let _ = storage.write(&key, data).await;
            }
        }
    }

    Ok(())
}

/// RAII guard to clean up temporary directory.
struct TempDirGuard(PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let path = self.0.clone();
        // Spawn blocking cleanup to avoid blocking async runtime
        std::thread::spawn(move || {
            let _ = std::fs::remove_dir_all(path);
        });
    }
}
