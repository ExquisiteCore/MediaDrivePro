use chrono::Utc;
use mdp_common::error::AppError;
use sea_orm::*;
use uuid::Uuid;

use crate::entity::{room_members, rooms, users};

#[derive(Debug, serde::Serialize)]
pub struct RoomInfo {
    pub id: Uuid,
    pub host_id: Uuid,
    pub host_name: String,
    pub name: String,
    pub invite_code: String,
    pub status: String,
    pub current_file_id: Option<Uuid>,
    pub current_time: f64,
    pub max_members: i32,
    pub member_count: i64,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, serde::Serialize)]
pub struct MemberInfo {
    pub user_id: Uuid,
    pub username: String,
    pub avatar: Option<String>,
    pub role: String,
}

#[derive(Debug, serde::Serialize)]
pub struct RoomDetail {
    pub room: RoomInfo,
    pub members: Vec<MemberInfo>,
}

pub struct RoomService;

impl RoomService {
    pub async fn create(
        db: &DatabaseConnection,
        host_id: Uuid,
        name: &str,
        max_members: Option<i32>,
    ) -> Result<RoomInfo, AppError> {
        let now = Utc::now();
        let room_id = Uuid::new_v4();
        let invite_code = generate_invite_code();

        let room = rooms::ActiveModel {
            id: Set(room_id),
            host_id: Set(host_id),
            name: Set(name.to_string()),
            invite_code: Set(invite_code),
            status: Set("waiting".to_string()),
            current_file_id: Set(None),
            current_time: Set(0.0),
            max_members: Set(max_members.unwrap_or(20)),
            created_at: Set(now),
            updated_at: Set(now),
        };
        let room = room.insert(db).await?;

        // Add host as member
        let member = room_members::ActiveModel {
            room_id: Set(room_id),
            user_id: Set(host_id),
            role: Set("host".to_string()),
            joined_at: Set(now),
        };
        member.insert(db).await?;

        // Get host username
        let host = users::Entity::find_by_id(host_id)
            .one(db)
            .await?
            .ok_or(AppError::NotFound("用户不存在".to_string()))?;

        Ok(RoomInfo {
            id: room.id,
            host_id: room.host_id,
            host_name: host.username,
            name: room.name,
            invite_code: room.invite_code,
            status: room.status,
            current_file_id: room.current_file_id,
            current_time: room.current_time,
            max_members: room.max_members,
            member_count: 1,
            created_at: room.created_at,
        })
    }

    pub async fn get(db: &DatabaseConnection, room_id: Uuid) -> Result<RoomInfo, AppError> {
        let room = rooms::Entity::find_by_id(room_id)
            .one(db)
            .await?
            .ok_or(AppError::NotFound("房间不存在".to_string()))?;

        let member_count = room_members::Entity::find()
            .filter(room_members::Column::RoomId.eq(room_id))
            .count(db)
            .await?;

        let host = users::Entity::find_by_id(room.host_id)
            .one(db)
            .await?
            .map(|u| u.username)
            .unwrap_or_default();

        Ok(RoomInfo {
            id: room.id,
            host_id: room.host_id,
            host_name: host,
            name: room.name,
            invite_code: room.invite_code,
            status: room.status,
            current_file_id: room.current_file_id,
            current_time: room.current_time,
            max_members: room.max_members,
            member_count: member_count as i64,
            created_at: room.created_at,
        })
    }

    pub async fn get_detail(
        db: &DatabaseConnection,
        room_id: Uuid,
    ) -> Result<RoomDetail, AppError> {
        let room = Self::get(db, room_id).await?;
        let members = Self::list_members(db, room_id).await?;
        Ok(RoomDetail { room, members })
    }

    pub async fn list_by_user(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Vec<RoomInfo>, AppError> {
        // Find room IDs where user is a member
        let member_rows = room_members::Entity::find()
            .filter(room_members::Column::UserId.eq(user_id))
            .all(db)
            .await?;

        let room_ids: Vec<Uuid> = member_rows.iter().map(|m| m.room_id).collect();
        if room_ids.is_empty() {
            return Ok(vec![]);
        }

        let room_rows = rooms::Entity::find()
            .filter(rooms::Column::Id.is_in(room_ids))
            .filter(rooms::Column::Status.ne("ended"))
            .order_by_desc(rooms::Column::CreatedAt)
            .all(db)
            .await?;

        let mut result = Vec::new();
        for room in room_rows {
            let member_count = room_members::Entity::find()
                .filter(room_members::Column::RoomId.eq(room.id))
                .count(db)
                .await?;

            let host_name = users::Entity::find_by_id(room.host_id)
                .one(db)
                .await?
                .map(|u| u.username)
                .unwrap_or_default();

            result.push(RoomInfo {
                id: room.id,
                host_id: room.host_id,
                host_name,
                name: room.name,
                invite_code: room.invite_code,
                status: room.status,
                current_file_id: room.current_file_id,
                current_time: room.current_time,
                max_members: room.max_members,
                member_count: member_count as i64,
                created_at: room.created_at,
            });
        }

        Ok(result)
    }

    pub async fn get_by_invite(
        db: &DatabaseConnection,
        invite_code: &str,
    ) -> Result<RoomInfo, AppError> {
        let room = rooms::Entity::find()
            .filter(rooms::Column::InviteCode.eq(invite_code))
            .one(db)
            .await?
            .ok_or(AppError::NotFound("邀请码无效".to_string()))?;

        if room.status == "ended" {
            return Err(AppError::Validation("房间已关闭".to_string()));
        }

        let member_count = room_members::Entity::find()
            .filter(room_members::Column::RoomId.eq(room.id))
            .count(db)
            .await?;

        let host_name = users::Entity::find_by_id(room.host_id)
            .one(db)
            .await?
            .map(|u| u.username)
            .unwrap_or_default();

        Ok(RoomInfo {
            id: room.id,
            host_id: room.host_id,
            host_name,
            name: room.name,
            invite_code: room.invite_code,
            status: room.status,
            current_file_id: room.current_file_id,
            current_time: room.current_time,
            max_members: room.max_members,
            member_count: member_count as i64,
            created_at: room.created_at,
        })
    }

    pub async fn join(
        db: &DatabaseConnection,
        room_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let room = rooms::Entity::find_by_id(room_id)
            .one(db)
            .await?
            .ok_or(AppError::NotFound("房间不存在".to_string()))?;

        if room.status == "ended" {
            return Err(AppError::Validation("房间已关闭".to_string()));
        }

        // Check if already a member
        let existing = room_members::Entity::find()
            .filter(room_members::Column::RoomId.eq(room_id))
            .filter(room_members::Column::UserId.eq(user_id))
            .one(db)
            .await?;

        if existing.is_some() {
            return Ok(()); // Already a member
        }

        // Check member limit
        let count = room_members::Entity::find()
            .filter(room_members::Column::RoomId.eq(room_id))
            .count(db)
            .await?;

        if count as i32 >= room.max_members {
            return Err(AppError::Validation("房间已满".to_string()));
        }

        let member = room_members::ActiveModel {
            room_id: Set(room_id),
            user_id: Set(user_id),
            role: Set("member".to_string()),
            joined_at: Set(Utc::now()),
        };
        member.insert(db).await?;

        Ok(())
    }

    pub async fn leave(
        db: &DatabaseConnection,
        room_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let member = room_members::Entity::find()
            .filter(room_members::Column::RoomId.eq(room_id))
            .filter(room_members::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(AppError::NotFound("不是房间成员".to_string()))?;

        member.delete(db).await?;
        Ok(())
    }

    pub async fn close(
        db: &DatabaseConnection,
        room_id: Uuid,
        host_id: Uuid,
    ) -> Result<(), AppError> {
        let room = rooms::Entity::find_by_id(room_id)
            .one(db)
            .await?
            .ok_or(AppError::NotFound("房间不存在".to_string()))?;

        if room.host_id != host_id {
            return Err(AppError::Forbidden);
        }

        let mut active: rooms::ActiveModel = room.into();
        active.status = Set("ended".to_string());
        active.updated_at = Set(Utc::now());
        active.update(db).await?;

        Ok(())
    }

    pub async fn update_playback(
        db: &DatabaseConnection,
        room_id: Uuid,
        file_id: Option<Uuid>,
        time: Option<f64>,
        status: Option<&str>,
    ) -> Result<rooms::Model, AppError> {
        let room = rooms::Entity::find_by_id(room_id)
            .one(db)
            .await?
            .ok_or(AppError::NotFound("房间不存在".to_string()))?;

        let mut active: rooms::ActiveModel = room.into();

        if let Some(fid) = file_id {
            active.current_file_id = Set(Some(fid));
            active.current_time = Set(0.0);
            active.status = Set("waiting".to_string());
        }
        if let Some(t) = time {
            active.current_time = Set(t);
        }
        if let Some(s) = status {
            active.status = Set(s.to_string());
        }

        active.updated_at = Set(Utc::now());
        let updated = active.update(db).await?;
        Ok(updated)
    }

    pub async fn list_members(
        db: &DatabaseConnection,
        room_id: Uuid,
    ) -> Result<Vec<MemberInfo>, AppError> {
        let members = room_members::Entity::find()
            .filter(room_members::Column::RoomId.eq(room_id))
            .all(db)
            .await?;

        let mut result = Vec::new();
        for m in members {
            let user = users::Entity::find_by_id(m.user_id).one(db).await?;
            if let Some(u) = user {
                result.push(MemberInfo {
                    user_id: u.id,
                    username: u.username,
                    avatar: u.avatar.clone(),
                    role: m.role,
                });
            }
        }

        Ok(result)
    }

    /// Check if a user is a member of the room
    pub async fn is_member(
        db: &DatabaseConnection,
        room_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, AppError> {
        let existing = room_members::Entity::find()
            .filter(room_members::Column::RoomId.eq(room_id))
            .filter(room_members::Column::UserId.eq(user_id))
            .one(db)
            .await?;
        Ok(existing.is_some())
    }

    /// Check if user is the host
    pub async fn is_host(
        db: &DatabaseConnection,
        room_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, AppError> {
        let room = rooms::Entity::find_by_id(room_id)
            .one(db)
            .await?
            .ok_or(AppError::NotFound("房间不存在".to_string()))?;
        Ok(room.host_id == user_id)
    }
}

fn generate_invite_code() -> String {
    let id = Uuid::new_v4();
    let bytes = id.as_bytes();
    hex::encode(&bytes[..4]) // 8 hex chars
}
