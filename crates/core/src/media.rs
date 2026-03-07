use chrono::Utc;
use mdp_common::{config::TmdbConfig, error::AppError};
use regex::Regex;
use sea_orm::*;
use uuid::Uuid;

use crate::entity::media_info;

#[derive(Debug, serde::Serialize)]
pub struct MediaInfoDto {
    pub id: Uuid,
    pub file_id: Uuid,
    pub media_type: String,
    pub title: Option<String>,
    pub season: Option<i32>,
    pub episode: Option<i32>,
    pub tmdb_id: Option<i32>,
    pub poster_url: Option<String>,
    pub overview: Option<String>,
    pub year: Option<i32>,
    pub duration: Option<i32>,
    pub resolution: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
}

impl From<media_info::Model> for MediaInfoDto {
    fn from(m: media_info::Model) -> Self {
        Self {
            id: m.id,
            file_id: m.file_id,
            media_type: m.media_type,
            title: m.title,
            season: m.season,
            episode: m.episode,
            tmdb_id: m.tmdb_id,
            poster_url: m.poster_url,
            overview: m.overview,
            year: m.year,
            duration: m.duration,
            resolution: m.resolution,
            created_at: m.created_at,
        }
    }
}

#[derive(Debug)]
pub struct ParsedMedia {
    pub title: String,
    pub season: Option<i32>,
    pub episode: Option<i32>,
    pub year: Option<i32>,
    pub quality: Option<String>,
}

pub struct MediaService;

impl MediaService {
    pub async fn get_by_file(
        db: &DatabaseConnection,
        file_id: Uuid,
    ) -> Result<Option<MediaInfoDto>, AppError> {
        let info = media_info::Entity::find()
            .filter(media_info::Column::FileId.eq(file_id))
            .one(db)
            .await?;
        Ok(info.map(MediaInfoDto::from))
    }

    /// Parse filename, optionally query TMDB, and upsert media_info.
    pub async fn scan(
        db: &DatabaseConnection,
        tmdb_config: &TmdbConfig,
        file_name: &str,
        file_id: Uuid,
    ) -> Result<MediaInfoDto, AppError> {
        let parsed = parse_filename(file_name);

        let mut title = Some(parsed.title.clone());
        let mut year = parsed.year;
        let season = parsed.season;
        let episode = parsed.episode;
        let mut tmdb_id = None;
        let mut poster_url = None;
        let mut overview = None;
        let mut media_type = if parsed.season.is_some() || parsed.episode.is_some() {
            "tv"
        } else {
            "movie"
        }
        .to_string();

        // Try TMDB if API key is configured
        if !tmdb_config.api_key.is_empty() {
            if let Ok(Some(result)) =
                search_tmdb(tmdb_config, &parsed.title, parsed.year).await
            {
                title = Some(result.title);
                year = result.year.or(year);
                tmdb_id = Some(result.tmdb_id);
                poster_url = result.poster_url;
                overview = result.overview;
                media_type = result.media_type;
            }
        }

        // Upsert media_info
        let existing = media_info::Entity::find()
            .filter(media_info::Column::FileId.eq(file_id))
            .one(db)
            .await?;

        let model = if let Some(existing) = existing {
            let mut active: media_info::ActiveModel = existing.into();
            active.media_type = Set(media_type);
            active.title = Set(title);
            active.season = Set(season);
            active.episode = Set(episode);
            active.year = Set(year);
            active.tmdb_id = Set(tmdb_id);
            active.poster_url = Set(poster_url);
            active.overview = Set(overview);
            active.update(db).await?
        } else {
            let info = media_info::ActiveModel {
                id: Set(Uuid::new_v4()),
                file_id: Set(file_id),
                media_type: Set(media_type),
                title: Set(title),
                season: Set(season),
                episode: Set(episode),
                tmdb_id: Set(tmdb_id),
                poster_url: Set(poster_url),
                overview: Set(overview),
                year: Set(year),
                duration: Set(None),
                resolution: Set(parsed.quality),
                created_at: Set(Utc::now()),
            };
            info.insert(db).await?
        };

        Ok(model.into())
    }
}

/// Parse common media file name patterns.
pub fn parse_filename(name: &str) -> ParsedMedia {
    // Remove file extension
    let name = name.rsplit_once('.').map(|(n, _)| n).unwrap_or(name);

    // Pattern 1: TV show "Title.S01E03.1080p.BluRay" or "Title S01E03"
    let tv_re = Regex::new(r"(?i)(.+?)[.\s_-]+S(\d{1,2})E(\d{1,3})").unwrap();
    if let Some(caps) = tv_re.captures(name) {
        let title = clean_title(&caps[1]);
        let season = caps[2].parse().ok();
        let episode = caps[3].parse().ok();
        let quality = extract_quality(name);
        let year = extract_year(name);
        return ParsedMedia {
            title,
            season,
            episode,
            year,
            quality,
        };
    }

    // Pattern 2: Anime "[SubGroup] Title - 03 (1080p)" or "[SubGroup] Title - 03"
    let anime_re = Regex::new(r"^\[.+?\]\s*(.+?)\s*-\s*(\d{1,3})").unwrap();
    if let Some(caps) = anime_re.captures(name) {
        let title = clean_title(&caps[1]);
        let episode = caps[2].parse().ok();
        let quality = extract_quality(name);
        return ParsedMedia {
            title,
            season: Some(1),
            episode,
            year: None,
            quality,
        };
    }

    // Pattern 3: Movie "Movie.Name.2024.720p.BluRay" or "Movie Name (2024)"
    let movie_year_re = Regex::new(r"(.+?)[.\s_(-]+(\d{4})[)\s._-]").unwrap();
    if let Some(caps) = movie_year_re.captures(name) {
        let title = clean_title(&caps[1]);
        let year = caps[2].parse().ok();
        let quality = extract_quality(name);
        return ParsedMedia {
            title,
            season: None,
            episode: None,
            year,
            quality,
        };
    }

    // Fallback: use whole name as title
    ParsedMedia {
        title: clean_title(name),
        season: None,
        episode: None,
        year: None,
        quality: extract_quality(name),
    }
}

fn clean_title(raw: &str) -> String {
    raw.replace('.', " ")
        .replace('_', " ")
        .trim()
        .to_string()
}

fn extract_quality(name: &str) -> Option<String> {
    let quality_re = Regex::new(r"(?i)(4K|2160p|1080p|720p|480p|SD)").unwrap();
    quality_re
        .captures(name)
        .map(|c| c[1].to_uppercase())
}

fn extract_year(name: &str) -> Option<i32> {
    let year_re = Regex::new(r"[.\s(](\d{4})[.\s)]").unwrap();
    year_re.captures(name).and_then(|c| c[1].parse().ok())
}

#[derive(Debug)]
struct TmdbResult {
    title: String,
    year: Option<i32>,
    tmdb_id: i32,
    poster_url: Option<String>,
    overview: Option<String>,
    media_type: String,
}

async fn search_tmdb(
    config: &TmdbConfig,
    title: &str,
    year: Option<i32>,
) -> Result<Option<TmdbResult>, AppError> {
    let client = reqwest::Client::new();

    let url = format!("{}/search/multi", config.base_url);

    let mut query = vec![
        ("api_key", config.api_key.clone()),
        ("query", title.to_string()),
        ("language", config.language.clone()),
    ];
    if let Some(y) = year {
        query.push(("year", y.to_string()));
    }

    let resp = client
        .get(&url)
        .query(&query)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("TMDB 请求失败: {e}")))?;

    if !resp.status().is_success() {
        return Ok(None);
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("TMDB 响应解析失败: {e}")))?;

    let results = json["results"].as_array();
    let first = match results.and_then(|r| r.first()) {
        Some(r) => r,
        None => return Ok(None),
    };

    let media_type = first["media_type"].as_str().unwrap_or("movie");
    let tmdb_id = first["id"].as_i64().unwrap_or(0) as i32;

    let title = first["title"]
        .as_str()
        .or_else(|| first["name"].as_str())
        .unwrap_or("")
        .to_string();

    let year_str = first["release_date"]
        .as_str()
        .or_else(|| first["first_air_date"].as_str())
        .unwrap_or("");
    let year = year_str.get(..4).and_then(|y| y.parse().ok());

    let poster_path = first["poster_path"].as_str();
    let poster_url = poster_path.map(|p| format!("https://image.tmdb.org/t/p/w500{p}"));

    let overview = first["overview"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    Ok(Some(TmdbResult {
        title,
        year,
        tmdb_id,
        poster_url,
        overview,
        media_type: media_type.to_string(),
    }))
}
