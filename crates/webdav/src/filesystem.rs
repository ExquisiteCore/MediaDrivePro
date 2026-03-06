use chrono::Utc;
use dav_server::davpath::DavPath;
use dav_server::fs::*;
use futures::FutureExt;
use opendal::Operator;
use sea_orm::*;
use std::io;
use uuid::Uuid;

use mdp_core::entity::{files, folders};

/// A WebDAV filesystem backed by SeaORM (metadata) + OpenDAL (data).
#[derive(Clone)]
pub struct MdpDavFs {
    db: DatabaseConnection,
    storage: Operator,
    user_id: Uuid,
    storage_backend: String,
}

impl MdpDavFs {
    pub fn new(
        db: DatabaseConnection,
        storage: Operator,
        user_id: Uuid,
        storage_backend: String,
    ) -> Self {
        Self {
            db,
            storage,
            user_id,
            storage_backend,
        }
    }
}

/// Resolved path: either root, a folder, or a file.
enum ResolvedPath {
    Root,
    Folder(folders::Model),
    File(files::Model),
    NotFound,
}

/// Metadata wrapper for files and folders.
#[derive(Clone)]
struct MdpMetaData {
    is_dir: bool,
    len: u64,
    modified: std::time::SystemTime,
    created: std::time::SystemTime,
    name: String,
}

impl DavMetaData for MdpMetaData {
    fn len(&self) -> u64 {
        self.len
    }
    fn modified(&self) -> FsResult<std::time::SystemTime> {
        Ok(self.modified)
    }
    fn is_dir(&self) -> bool {
        self.is_dir
    }
    fn created(&self) -> FsResult<std::time::SystemTime> {
        Ok(self.created)
    }
}

/// Directory entry for listing.
struct MdpDirEntry {
    meta: MdpMetaData,
    name: String,
}

impl DavDirEntry for MdpDirEntry {
    fn name(&self) -> Vec<u8> {
        self.name.clone().into_bytes()
    }
    fn metadata(&self) -> FsFuture<Box<dyn DavMetaData>> {
        let meta = self.meta.clone();
        async move { Ok(Box::new(meta) as Box<dyn DavMetaData>) }.boxed()
    }
}

/// An open file for reading/writing.
struct MdpOpenFile {
    data: Vec<u8>,
    pos: usize,
}

impl DavFile for MdpOpenFile {
    fn metadata(&mut self) -> FsFuture<Box<dyn DavMetaData>> {
        let len = self.data.len() as u64;
        async move {
            Ok(Box::new(MdpMetaData {
                is_dir: false,
                len,
                modified: std::time::SystemTime::now(),
                created: std::time::SystemTime::now(),
                name: String::new(),
            }) as Box<dyn DavMetaData>)
        }
        .boxed()
    }

    fn write_buf(&mut self, buf: Box<dyn bytes::Buf + Send>) -> FsFuture<()> {
        let chunk = buf.chunk().to_vec();
        self.data.extend_from_slice(&chunk);
        async { Ok(()) }.boxed()
    }

    fn write_bytes(&mut self, buf: bytes::Bytes) -> FsFuture<()> {
        self.data.extend_from_slice(&buf);
        async { Ok(()) }.boxed()
    }

    fn read_bytes(&mut self, count: usize) -> FsFuture<bytes::Bytes> {
        let end = (self.pos + count).min(self.data.len());
        let slice = &self.data[self.pos..end];
        let bytes = bytes::Bytes::copy_from_slice(slice);
        self.pos = end;
        async move { Ok(bytes) }.boxed()
    }

    fn seek(&mut self, pos: io::SeekFrom) -> FsFuture<u64> {
        let new_pos = match pos {
            io::SeekFrom::Start(n) => n as i64,
            io::SeekFrom::End(n) => self.data.len() as i64 + n,
            io::SeekFrom::Current(n) => self.pos as i64 + n,
        };
        if new_pos < 0 {
            return async { Err(FsError::GeneralFailure) }.boxed();
        }
        self.pos = new_pos as usize;
        async move { Ok(new_pos as u64) }.boxed()
    }

    fn flush(&mut self) -> FsFuture<()> {
        async { Ok(()) }.boxed()
    }
}

fn to_system_time(dt: chrono::DateTime<Utc>) -> std::time::SystemTime {
    std::time::UNIX_EPOCH + std::time::Duration::from_secs(dt.timestamp() as u64)
}

impl MdpDavFs {
    /// Resolve a WebDAV path to our DB entities.
    /// Path components: /segment1/segment2/.../segmentN
    async fn resolve_path(&self, path: &DavPath) -> ResolvedPath {
        let path_str = path.as_url_string();
        let path_str = path_str.trim_start_matches('/');
        let path_str = percent_decode(path_str);

        if path_str.is_empty() || path_str == "/" {
            return ResolvedPath::Root;
        }

        let segments: Vec<&str> = path_str.trim_end_matches('/').split('/').collect();
        let is_dir_hint = path_str.ends_with('/');

        // Walk path segments: each segment is a folder name, except possibly the last
        let mut parent_id: Option<Uuid> = None;

        for (i, segment) in segments.iter().enumerate() {
            if segment.is_empty() {
                continue;
            }
            let is_last = i == segments.len() - 1;

            // Try to find a folder with this name
            let mut folder_query = folders::Entity::find()
                .filter(folders::Column::UserId.eq(self.user_id))
                .filter(folders::Column::Name.eq(*segment));

            if let Some(pid) = parent_id {
                folder_query = folder_query.filter(folders::Column::ParentId.eq(pid));
            } else {
                folder_query = folder_query.filter(folders::Column::ParentId.is_null());
            }

            if let Ok(Some(folder)) = folder_query.one(&self.db).await {
                if is_last {
                    return ResolvedPath::Folder(folder);
                }
                parent_id = Some(folder.id);
                continue;
            }

            // If last segment and not a folder, try file
            if is_last && !is_dir_hint {
                let mut file_query = files::Entity::find()
                    .filter(files::Column::UserId.eq(self.user_id))
                    .filter(files::Column::Name.eq(*segment))
                    .filter(files::Column::Status.ne("deleted"));

                if let Some(pid) = parent_id {
                    file_query = file_query.filter(files::Column::FolderId.eq(pid));
                } else {
                    file_query = file_query.filter(files::Column::FolderId.is_null());
                }

                if let Ok(Some(file)) = file_query.one(&self.db).await {
                    return ResolvedPath::File(file);
                }
            }

            return ResolvedPath::NotFound;
        }

        ResolvedPath::Root
    }

    /// Get the parent folder_id for a given path (the parent of the last segment).
    async fn resolve_parent(&self, path: &DavPath) -> Result<Option<Uuid>, FsError> {
        let path_str = path.as_url_string();
        let path_str = path_str.trim_start_matches('/');
        let path_str = percent_decode(path_str);
        let segments: Vec<&str> = path_str
            .trim_end_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        if segments.len() <= 1 {
            return Ok(None); // parent is root
        }

        // Walk all but the last segment
        let mut parent_id: Option<Uuid> = None;
        for segment in &segments[..segments.len() - 1] {
            let mut query = folders::Entity::find()
                .filter(folders::Column::UserId.eq(self.user_id))
                .filter(folders::Column::Name.eq(*segment));

            if let Some(pid) = parent_id {
                query = query.filter(folders::Column::ParentId.eq(pid));
            } else {
                query = query.filter(folders::Column::ParentId.is_null());
            }

            let folder = query
                .one(&self.db)
                .await
                .map_err(|_| FsError::GeneralFailure)?
                .ok_or(FsError::NotFound)?;
            parent_id = Some(folder.id);
        }
        Ok(parent_id)
    }

    /// Get the last segment (file/folder name) from a path.
    fn last_segment(path: &DavPath) -> String {
        let path_str = path.as_url_string();
        let path_str = path_str.trim_start_matches('/');
        let decoded = percent_decode(path_str);
        decoded
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("")
            .to_string()
    }
}

fn percent_decode(s: &str) -> String {
    urlencoding_decode(s)
}

fn urlencoding_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            let h = chars.next().unwrap_or(b'0');
            let l = chars.next().unwrap_or(b'0');
            let byte =
                hex_val(h) * 16 + hex_val(l);
            result.push(byte as char);
        } else if b == b'+' {
            result.push(' ');
        } else {
            result.push(b as char);
        }
    }
    result
}

fn hex_val(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => 0,
    }
}

impl DavFileSystem for MdpDavFs {
    fn open<'a>(
        &'a self,
        path: &'a DavPath,
        options: OpenOptions,
    ) -> FsFuture<Box<dyn DavFile>> {
        let path = path.clone();
        let fs = self.clone();

        async move {
            let name = Self::last_segment(&path);

            if options.write && options.create {
                // PUT (create/overwrite file)
                let parent_id = fs.resolve_parent(&path).await?;

                // Check if file exists
                let mut query = files::Entity::find()
                    .filter(files::Column::UserId.eq(fs.user_id))
                    .filter(files::Column::Name.eq(&name))
                    .filter(files::Column::Status.ne("deleted"));
                if let Some(pid) = parent_id {
                    query = query.filter(files::Column::FolderId.eq(pid));
                } else {
                    query = query.filter(files::Column::FolderId.is_null());
                }

                if let Ok(Some(existing)) = query.one(&fs.db).await {
                    // Delete existing file from storage
                    fs.storage
                        .delete(&existing.storage_key)
                        .await
                        .map_err(|_| FsError::GeneralFailure)?;
                    existing
                        .delete(&fs.db)
                        .await
                        .map_err(|_| FsError::GeneralFailure)?;
                }

                // Return an empty file that will accumulate write data.
                // The actual storage write happens in flush/close — but dav-server
                // expects us to handle it. We use a WriteFile wrapper.
                Ok(Box::new(MdpWriteFile {
                    db: fs.db.clone(),
                    storage: fs.storage.clone(),
                    user_id: fs.user_id,
                    folder_id: parent_id,
                    file_name: name,
                    storage_backend: fs.storage_backend.clone(),
                    data: Vec::new(),
                }) as Box<dyn DavFile>)
            } else {
                // GET (read file)
                match fs.resolve_path(&path).await {
                    ResolvedPath::File(file) => {
                        let data = fs
                            .storage
                            .read(&file.storage_key)
                            .await
                            .map_err(|_| FsError::GeneralFailure)?
                            .to_vec();
                        Ok(Box::new(MdpOpenFile { data, pos: 0 }) as Box<dyn DavFile>)
                    }
                    _ => Err(FsError::NotFound),
                }
            }
        }
        .boxed()
    }

    fn read_dir<'a>(
        &'a self,
        path: &'a DavPath,
        _meta: ReadDirMeta,
    ) -> FsFuture<FsStream> {
        let path = path.clone();
        let fs = self.clone();

        async move {
            let folder_id = match fs.resolve_path(&path).await {
                ResolvedPath::Root => None,
                ResolvedPath::Folder(f) => Some(f.id),
                _ => return Err(FsError::NotFound),
            };

            let mut entries: Vec<Box<dyn DavDirEntry>> = Vec::new();

            // Sub-folders
            let mut fq = folders::Entity::find()
                .filter(folders::Column::UserId.eq(fs.user_id));
            if let Some(fid) = folder_id {
                fq = fq.filter(folders::Column::ParentId.eq(fid));
            } else {
                fq = fq.filter(folders::Column::ParentId.is_null());
            }
            if let Ok(folders_list) = fq.all(&fs.db).await {
                for f in folders_list {
                    entries.push(Box::new(MdpDirEntry {
                        name: f.name.clone(),
                        meta: MdpMetaData {
                            is_dir: true,
                            len: 0,
                            modified: to_system_time(f.updated_at),
                            created: to_system_time(f.created_at),
                            name: f.name,
                        },
                    }));
                }
            }

            // Files
            let mut fileq = files::Entity::find()
                .filter(files::Column::UserId.eq(fs.user_id))
                .filter(files::Column::Status.ne("deleted"));
            if let Some(fid) = folder_id {
                fileq = fileq.filter(files::Column::FolderId.eq(fid));
            } else {
                fileq = fileq.filter(files::Column::FolderId.is_null());
            }
            if let Ok(files_list) = fileq.all(&fs.db).await {
                for f in files_list {
                    entries.push(Box::new(MdpDirEntry {
                        name: f.name.clone(),
                        meta: MdpMetaData {
                            is_dir: false,
                            len: f.size as u64,
                            modified: to_system_time(f.updated_at),
                            created: to_system_time(f.created_at),
                            name: f.name,
                        },
                    }));
                }
            }

            let stream = futures::stream::iter(
                entries.into_iter().map(Ok),
            );
            Ok(Box::pin(stream) as FsStream)
        }
        .boxed()
    }

    fn metadata<'a>(&'a self, path: &'a DavPath) -> FsFuture<Box<dyn DavMetaData>> {
        let path = path.clone();
        let fs = self.clone();

        async move {
            match fs.resolve_path(&path).await {
                ResolvedPath::Root => Ok(Box::new(MdpMetaData {
                    is_dir: true,
                    len: 0,
                    modified: std::time::SystemTime::now(),
                    created: std::time::SystemTime::now(),
                    name: String::new(),
                }) as Box<dyn DavMetaData>),
                ResolvedPath::Folder(f) => Ok(Box::new(MdpMetaData {
                    is_dir: true,
                    len: 0,
                    modified: to_system_time(f.updated_at),
                    created: to_system_time(f.created_at),
                    name: f.name,
                }) as Box<dyn DavMetaData>),
                ResolvedPath::File(f) => Ok(Box::new(MdpMetaData {
                    is_dir: false,
                    len: f.size as u64,
                    modified: to_system_time(f.updated_at),
                    created: to_system_time(f.created_at),
                    name: f.name,
                }) as Box<dyn DavMetaData>),
                ResolvedPath::NotFound => Err(FsError::NotFound),
            }
        }
        .boxed()
    }

    fn create_dir<'a>(&'a self, path: &'a DavPath) -> FsFuture<()> {
        let path = path.clone();
        let fs = self.clone();

        async move {
            let name = Self::last_segment(&path);
            if name.is_empty() {
                return Err(FsError::Forbidden);
            }
            let parent_id = fs.resolve_parent(&path).await?;

            let now = Utc::now();
            let folder = folders::ActiveModel {
                id: sea_orm::Set(Uuid::new_v4()),
                user_id: sea_orm::Set(fs.user_id),
                parent_id: sea_orm::Set(parent_id),
                name: sea_orm::Set(name),
                created_at: sea_orm::Set(now),
                updated_at: sea_orm::Set(now),
            };
            folder
                .insert(&fs.db)
                .await
                .map_err(|_| FsError::Exists)?;
            Ok(())
        }
        .boxed()
    }

    fn remove_dir<'a>(&'a self, path: &'a DavPath) -> FsFuture<()> {
        let path = path.clone();
        let fs = self.clone();

        async move {
            match fs.resolve_path(&path).await {
                ResolvedPath::Folder(f) => {
                    f.delete(&fs.db)
                        .await
                        .map_err(|_| FsError::GeneralFailure)?;
                    Ok(())
                }
                _ => Err(FsError::NotFound),
            }
        }
        .boxed()
    }

    fn remove_file<'a>(&'a self, path: &'a DavPath) -> FsFuture<()> {
        let path = path.clone();
        let fs = self.clone();

        async move {
            match fs.resolve_path(&path).await {
                ResolvedPath::File(file) => {
                    // Delete from storage
                    fs.storage
                        .delete(&file.storage_key)
                        .await
                        .map_err(|_| FsError::GeneralFailure)?;
                    // Mark as deleted
                    let mut active: files::ActiveModel = file.into();
                    active.status = sea_orm::Set("deleted".to_string());
                    active.updated_at = sea_orm::Set(Utc::now());
                    active
                        .update(&fs.db)
                        .await
                        .map_err(|_| FsError::GeneralFailure)?;
                    Ok(())
                }
                _ => Err(FsError::NotFound),
            }
        }
        .boxed()
    }

    fn rename<'a>(
        &'a self,
        from: &'a DavPath,
        to: &'a DavPath,
    ) -> FsFuture<()> {
        let from = from.clone();
        let to = to.clone();
        let fs = self.clone();

        async move {
            let new_name = Self::last_segment(&to);
            let new_parent = fs.resolve_parent(&to).await?;

            match fs.resolve_path(&from).await {
                ResolvedPath::Folder(folder) => {
                    let mut active: folders::ActiveModel = folder.into();
                    active.name = sea_orm::Set(new_name);
                    active.parent_id = sea_orm::Set(new_parent);
                    active.updated_at = sea_orm::Set(Utc::now());
                    active
                        .update(&fs.db)
                        .await
                        .map_err(|_| FsError::GeneralFailure)?;
                    Ok(())
                }
                ResolvedPath::File(file) => {
                    let mut active: files::ActiveModel = file.into();
                    active.name = sea_orm::Set(new_name);
                    active.folder_id = sea_orm::Set(new_parent);
                    active.updated_at = sea_orm::Set(Utc::now());
                    active
                        .update(&fs.db)
                        .await
                        .map_err(|_| FsError::GeneralFailure)?;
                    Ok(())
                }
                _ => Err(FsError::NotFound),
            }
        }
        .boxed()
    }

    fn copy<'a>(
        &'a self,
        from: &'a DavPath,
        to: &'a DavPath,
    ) -> FsFuture<()> {
        let from = from.clone();
        let to = to.clone();
        let fs = self.clone();

        async move {
            match fs.resolve_path(&from).await {
                ResolvedPath::File(src_file) => {
                    let new_name = Self::last_segment(&to);
                    let new_parent = fs.resolve_parent(&to).await?;

                    // Read source data
                    let data = fs
                        .storage
                        .read(&src_file.storage_key)
                        .await
                        .map_err(|_| FsError::GeneralFailure)?
                        .to_vec();

                    // Create new file
                    let file_id = Uuid::new_v4();
                    let storage_key =
                        mdp_storage::storage_key::file(fs.user_id, file_id);
                    fs.storage
                        .write(&storage_key, data)
                        .await
                        .map_err(|_| FsError::GeneralFailure)?;

                    let now = Utc::now();
                    let new_file = files::ActiveModel {
                        id: sea_orm::Set(file_id),
                        user_id: sea_orm::Set(fs.user_id),
                        folder_id: sea_orm::Set(new_parent),
                        name: sea_orm::Set(new_name),
                        storage_key: sea_orm::Set(storage_key),
                        size: sea_orm::Set(src_file.size),
                        content_type: sea_orm::Set(src_file.content_type.clone()),
                        hash_sha256: sea_orm::Set(src_file.hash_sha256.clone()),
                        storage_backend: sea_orm::Set(
                            src_file.storage_backend.clone(),
                        ),
                        status: sea_orm::Set("active".to_string()),
                        created_at: sea_orm::Set(now),
                        updated_at: sea_orm::Set(now),
                    };
                    new_file
                        .insert(&fs.db)
                        .await
                        .map_err(|_| FsError::GeneralFailure)?;
                    Ok(())
                }
                _ => Err(FsError::NotFound),
            }
        }
        .boxed()
    }
}

/// A write-only file handle. Accumulates data and writes to storage on flush.
struct MdpWriteFile {
    db: DatabaseConnection,
    storage: Operator,
    user_id: Uuid,
    folder_id: Option<Uuid>,
    file_name: String,
    storage_backend: String,
    data: Vec<u8>,
}

impl DavFile for MdpWriteFile {
    fn metadata(&mut self) -> FsFuture<Box<dyn DavMetaData>> {
        let len = self.data.len() as u64;
        async move {
            Ok(Box::new(MdpMetaData {
                is_dir: false,
                len,
                modified: std::time::SystemTime::now(),
                created: std::time::SystemTime::now(),
                name: String::new(),
            }) as Box<dyn DavMetaData>)
        }
        .boxed()
    }

    fn write_buf(&mut self, buf: Box<dyn bytes::Buf + Send>) -> FsFuture<()> {
        let chunk = buf.chunk().to_vec();
        self.data.extend_from_slice(&chunk);
        async { Ok(()) }.boxed()
    }

    fn write_bytes(&mut self, buf: bytes::Bytes) -> FsFuture<()> {
        self.data.extend_from_slice(&buf);
        async { Ok(()) }.boxed()
    }

    fn read_bytes(&mut self, _count: usize) -> FsFuture<bytes::Bytes> {
        async { Err(FsError::NotImplemented) }.boxed()
    }

    fn seek(&mut self, _pos: io::SeekFrom) -> FsFuture<u64> {
        async { Err(FsError::NotImplemented) }.boxed()
    }

    fn flush(&mut self) -> FsFuture<()> {
        // Write file to storage and DB
        let db = self.db.clone();
        let storage = self.storage.clone();
        let user_id = self.user_id;
        let folder_id = self.folder_id;
        let file_name = self.file_name.clone();
        let backend = self.storage_backend.clone();
        let data = std::mem::take(&mut self.data);

        async move {
            if data.is_empty() {
                return Ok(());
            }

            let file_id = Uuid::new_v4();
            let size = data.len() as i64;
            let storage_key = mdp_storage::storage_key::file(user_id, file_id);

            // Compute hash
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&data);
            let hash = hex::encode(hasher.finalize());

            // Guess content type
            let content_type = mime_from_name(&file_name);

            // Write to storage
            storage
                .write(&storage_key, data)
                .await
                .map_err(|_| FsError::GeneralFailure)?;

            let now = Utc::now();
            let file = files::ActiveModel {
                id: sea_orm::Set(file_id),
                user_id: sea_orm::Set(user_id),
                folder_id: sea_orm::Set(folder_id),
                name: sea_orm::Set(file_name),
                storage_key: sea_orm::Set(storage_key),
                size: sea_orm::Set(size),
                content_type: sea_orm::Set(content_type),
                hash_sha256: sea_orm::Set(hash),
                storage_backend: sea_orm::Set(backend),
                status: sea_orm::Set("active".to_string()),
                created_at: sea_orm::Set(now),
                updated_at: sea_orm::Set(now),
            };
            file.insert(&db)
                .await
                .map_err(|_| FsError::GeneralFailure)?;

            Ok(())
        }
        .boxed()
    }
}

fn mime_from_name(name: &str) -> String {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "txt" => "text/plain",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "gz" | "gzip" => "application/gzip",
        "tar" => "application/x-tar",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "mp4" => "video/mp4",
        "mkv" => "video/x-matroska",
        "avi" => "video/x-msvideo",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "flac" => "audio/flac",
        _ => "application/octet-stream",
    }
    .to_string()
}
