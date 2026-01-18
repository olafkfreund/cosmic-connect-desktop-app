//! File Sync Plugin
//!
//! Automatic file synchronization between connected desktops.
//!
//! ## Protocol Specification
//!
//! This plugin implements automatic file synchronization similar to Syncthing,
//! enabling users to keep specific folders synchronized across multiple desktops.
//!
//! ### Packet Types
//!
//! - `cconnect.filesync.config` - Sync folder configuration
//! - `cconnect.filesync.index` - File list with hashes and metadata
//! - `cconnect.filesync.transfer` - File data transfer (via payload)
//! - `cconnect.filesync.request` - Request file transfer
//! - `cconnect.filesync.conflict` - Conflict notification
//! - `cconnect.filesync.delete` - File deletion synchronization
//!
//! ### Capabilities
//!
//! - Incoming: `cconnect.filesync` - Can receive file sync operations
//! - Outgoing: `cconnect.filesync` - Can send file sync operations
//!
//! ### Use Cases
//!
//! - Keep work directories synchronized across machines
//! - Automatic backup to another desktop
//! - Collaborative file sharing between desktops
//! - Project synchronization for development
//!
//! ## Features
//!
//! - **Bidirectional Sync**: Automatic two-way synchronization
//! - **Real-time Watching**: inotify-based file system monitoring
//! - **Conflict Resolution**: Multiple strategies for handling conflicts
//! - **Selective Sync**: Ignore patterns and filters
//! - **File Versioning**: Keep previous versions of files
//! - **Delta Sync**: Only transfer changed parts (rsync algorithm)
//! - **Bandwidth Limiting**: Control network usage
//! - **Hash Comparison**: Fast content comparison with BLAKE3
//!
//! ## Conflict Resolution Strategies
//!
//! - **LastModifiedWins**: Use most recently modified file (default)
//! - **KeepBoth**: Rename conflicting file with timestamp
//! - **Manual**: Prompt user for resolution
//! - **SizeBased**: Keep larger file
//!
//! ## Implementation Status
//!
//! - [x] File system monitoring (notify integration)
//! - [x] BLAKE3 hashing for content comparison
//! - [x] Sync logic and plan generation
//! - [ ] File transfer implementation (upload/download)
//! - [ ] SQLite database for sync state (history)
//! - [ ] Delta sync algorithm (rsync-like)
//! - [ ] File versioning system
//! - [ ] Bandwidth limiting implementation

use crate::plugins::{Plugin, PluginFactory};
use crate::{Device, Packet, ProtocolError, Result};
use async_trait::async_trait;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};
use walkdir::WalkDir;

const PLUGIN_NAME: &str = "filesync";
const INCOMING_CAPABILITY: &str = "cconnect.filesync";
const OUTGOING_CAPABILITY: &str = "cconnect.filesync";

// File sync configuration constants
#[allow(dead_code)]
const MAX_FILE_SIZE_MB: u64 = 1024; // 1GB max file size
const DEFAULT_SCAN_INTERVAL_SECS: u64 = 60; // Scan every minute
const DEFAULT_VERSION_KEEP: usize = 5; // Keep 5 previous versions

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictStrategy {
    /// Use the most recently modified file (default)
    LastModifiedWins,
    /// Keep both files, rename with timestamp
    KeepBoth,
    /// Prompt user for manual resolution
    Manual,
    /// Keep larger file
    SizeBased,
}

impl Default for ConflictStrategy {
    fn default() -> Self {
        Self::LastModifiedWins
    }
}

impl ConflictStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LastModifiedWins => "last_modified_wins",
            Self::KeepBoth => "keep_both",
            Self::Manual => "manual",
            Self::SizeBased => "size_based",
        }
    }
}

/// Sync folder configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncFolder {
    /// Local path to sync
    pub local_path: PathBuf,

    /// Remote path on other device
    pub remote_path: PathBuf,

    /// Whether sync is enabled for this folder
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Bidirectional sync (if false, only push)
    #[serde(default = "default_true")]
    pub bidirectional: bool,

    /// Ignore patterns (gitignore-style)
    #[serde(default)]
    pub ignore_patterns: Vec<String>,

    /// Conflict resolution strategy
    #[serde(default)]
    pub conflict_strategy: ConflictStrategy,

    /// Enable file versioning
    #[serde(default = "default_true")]
    pub versioning: bool,

    /// Number of versions to keep
    #[serde(default = "default_version_keep")]
    pub version_keep: usize,

    /// Scan interval in seconds (0 = real-time watching only)
    #[serde(default = "default_scan_interval")]
    pub scan_interval_secs: u64,

    /// Bandwidth limit in KB/s (0 = unlimited)
    #[serde(default)]
    pub bandwidth_limit_kbps: u32,
}

fn default_true() -> bool {
    true
}

fn default_version_keep() -> usize {
    DEFAULT_VERSION_KEEP
}

fn default_scan_interval() -> u64 {
    DEFAULT_SCAN_INTERVAL_SECS
}

impl SyncFolder {
    pub fn validate(&self) -> Result<()> {
        if !self.local_path.exists() {
            return Err(ProtocolError::InvalidPacket(format!(
                "Local path does not exist: {}",
                self.local_path.display()
            )));
        }

        if !self.local_path.is_dir() {
            return Err(ProtocolError::InvalidPacket(format!(
                "Local path is not a directory: {}",
                self.local_path.display()
            )));
        }

        if self.version_keep == 0 && self.versioning {
            return Err(ProtocolError::InvalidPacket(
                "version_keep must be > 0 when versioning is enabled".to_string(),
            ));
        }

        Ok(())
    }
}

/// File metadata for sync index
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMetadata {
    /// Relative path from sync folder root
    pub path: PathBuf,

    /// File size in bytes
    pub size: u64,

    /// Last modified timestamp (milliseconds since epoch)
    pub modified: i64,

    /// BLAKE3 hash of file content
    pub hash: String,

    /// Whether this is a directory
    pub is_dir: bool,

    /// File permissions (Unix mode)
    #[serde(default)]
    pub permissions: Option<u32>,
}

/// Sync index containing all files in a folder
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncIndex {
    /// Folder identifier
    pub folder_id: String,

    /// Files in this folder
    pub files: Vec<FileMetadata>,

    /// Index generation timestamp (milliseconds since epoch)
    pub timestamp: i64,

    /// Total size of all files
    pub total_size: u64,

    /// Number of files
    pub file_count: usize,
}

/// File conflict information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileConflict {
    /// Folder identifier
    pub folder_id: String,

    /// File path
    pub path: PathBuf,

    /// Local file metadata
    pub local_metadata: FileMetadata,

    /// Remote file metadata
    pub remote_metadata: FileMetadata,

    /// Suggested resolution
    pub suggested_strategy: ConflictStrategy,

    /// Conflict detection timestamp (milliseconds since epoch)
    pub timestamp: i64,
}

/// Action to perform during synchronization
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncAction {
    Upload(PathBuf),
    Download(PathBuf),
    DeleteRemote(PathBuf),
    DeleteLocal(PathBuf),
    Conflict(FileConflict),
}

/// Synchronization plan
#[derive(Debug, Clone, Default)]
pub struct SyncPlan {
    pub actions: Vec<SyncAction>,
    pub stats: SyncStats,
}

#[derive(Debug, Clone, Default)]
pub struct SyncStats {
    pub files_to_upload: usize,
    pub files_to_download: usize,
    pub bytes_to_upload: u64,
    pub bytes_to_download: u64,
    pub conflicts: usize,
}

/// File sync plugin
pub struct FileSyncPlugin {
    /// Device ID this plugin is associated with
    device_id: Option<String>,

    /// Plugin enabled state
    enabled: bool,

    /// Configured sync folders
    sync_folders: HashMap<String, SyncFolder>,

    /// Current sync index by folder ID
    sync_indexes: HashMap<String, SyncIndex>,

    /// Pending conflicts
    pending_conflicts: Vec<FileConflict>,

    /// Active transfers (folder_id -> file_path)
    active_transfers: HashMap<String, Vec<PathBuf>>,

    /// File system watcher
    watcher: Option<RecommendedWatcher>,
}

impl FileSyncPlugin {
    /// Create new file sync plugin instance
    pub fn new() -> Self {
        Self {
            device_id: None,
            enabled: false,
            sync_folders: HashMap::new(),
            sync_indexes: HashMap::new(),
            pending_conflicts: Vec::new(),
            active_transfers: HashMap::new(),
            watcher: None,
        }
    }

    /// Add or update sync folder configuration
    pub fn configure_folder(&mut self, folder_id: String, config: SyncFolder) -> Result<()> {
        config.validate()?;

        info!(
            "Configuring sync folder '{}': {} -> {}",
            folder_id,
            config.local_path.display(),
            config.remote_path.display()
        );

        self.sync_folders.insert(folder_id.clone(), config.clone());

        // Start watching if plugin is enabled
        if self.enabled {
            if let Some(watcher) = &mut self.watcher {
                if let Err(e) = watcher.watch(&config.local_path, RecursiveMode::Recursive) {
                    warn!(
                        "Failed to watch folder {}: {}",
                        config.local_path.display(),
                        e
                    );
                } else {
                    info!("Started watching folder: {}", config.local_path.display());
                }
            }
        }

        // TODO: Trigger initial index generation

        Ok(())
    }

    /// Remove sync folder configuration
    pub fn remove_folder(&mut self, folder_id: &str) -> Result<()> {
        if let Some(config) = self.sync_folders.remove(folder_id) {
            // Clean up related data
            self.sync_indexes.remove(folder_id);
            self.active_transfers.remove(folder_id);
            self.pending_conflicts.retain(|c| c.folder_id != folder_id);

            info!("Removed sync folder '{}'", folder_id);

            // Stop file system watching for this folder
            if let Some(watcher) = &mut self.watcher {
                if let Err(e) = watcher.unwatch(&config.local_path) {
                    warn!(
                        "Failed to unwatch folder {}: {}",
                        config.local_path.display(),
                        e
                    );
                }
            }

            Ok(())
        } else {
            Err(ProtocolError::Plugin(format!(
                "Sync folder not found: {}",
                folder_id
            )))
        }
    }

    /// Compute BLAKE3 hash of a file
    fn compute_file_hash<P: AsRef<std::path::Path>>(path: P) -> Result<String> {
        let mut file = fs::File::open(path).map_err(|e| ProtocolError::Io(e))?;
        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0; 65536]; // 64KB buffer

        loop {
            let count = file.read(&mut buffer).map_err(|e| ProtocolError::Io(e))?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Generate sync index for a folder
    pub async fn generate_index(&self, folder_id: &str) -> Result<SyncIndex> {
        let config = self.sync_folders.get(folder_id).ok_or_else(|| {
            ProtocolError::Plugin(format!("Sync folder not found: {}", folder_id))
        })?;

        info!(
            "Generating sync index for folder '{}' at {}",
            folder_id,
            config.local_path.display()
        );

        let mut files = Vec::new();
        let mut total_size = 0;

        for entry in WalkDir::new(&config.local_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip the root folder itself
            if path == config.local_path {
                continue;
            }

            // Calculate relative path
            let relative_path = match path.strip_prefix(&config.local_path) {
                Ok(p) => p.to_path_buf(),
                Err(_) => continue,
            };

            // Basic ignore logic (TODO: Use robust glob matching)
            let path_str = relative_path.to_string_lossy();
            if config
                .ignore_patterns
                .iter()
                .any(|pattern| path_str.contains(pattern))
            {
                continue;
            }
            if path_str.contains(".git") || path_str.contains(".DS_Store") {
                continue;
            }

            let metadata = entry.metadata().map_err(|e| ProtocolError::Io(e.into()))?;
            let is_dir = metadata.is_dir();
            let size = metadata.len();
            let modified = metadata
                .modified()
                .unwrap_or(SystemTime::now())
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64;

            // Unix permissions (if applicable)
            #[cfg(unix)]
            let permissions = {
                use std::os::unix::fs::MetadataExt;
                Some(metadata.mode())
            };
            #[cfg(not(unix))]
            let permissions = None;

            let hash = if is_dir {
                String::new()
            } else {
                Self::compute_file_hash(path)?
            };

            if !is_dir {
                total_size += size;
            }

            files.push(FileMetadata {
                path: relative_path,
                size,
                modified,
                hash,
                is_dir,
                permissions,
            });
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        debug!(
            "Generated index for {}: {} files, {} bytes",
            folder_id,
            files.len(),
            total_size
        );

        let index = SyncIndex {
            folder_id: folder_id.to_string(),
            file_count: files.len(),
            files,
            timestamp,
            total_size,
        };

        Ok(index)
    }

    /// Create a synchronization plan by comparing local and remote indexes
    pub fn create_sync_plan(
        &self,
        folder_id: &str,
        local_index: &SyncIndex,
        remote_index: &SyncIndex,
    ) -> SyncPlan {
        let mut plan = SyncPlan::default();

        let config = match self.sync_folders.get(folder_id) {
            Some(c) => c,
            None => return plan,
        };

        // Efficient lookups
        let local_map: HashMap<&PathBuf, &FileMetadata> =
            local_index.files.iter().map(|f| (&f.path, f)).collect();
        let remote_map: HashMap<&PathBuf, &FileMetadata> =
            remote_index.files.iter().map(|f| (&f.path, f)).collect();

        // 1. Check local files (Uploads / Conflicts)
        for (path, local_file) in &local_map {
            match remote_map.get(path) {
                Some(remote_file) => {
                    // File exists on both sides
                    if local_file.hash != remote_file.hash {
                        // Content differs, check timestamps
                        if local_file.modified > remote_file.modified {
                            // Local is newer -> Upload
                            plan.actions.push(SyncAction::Upload(path.to_path_buf()));
                            plan.stats.files_to_upload += 1;
                            plan.stats.bytes_to_upload += local_file.size;
                        } else if remote_file.modified > local_file.modified {
                            // Remote is newer -> Download
                            plan.actions.push(SyncAction::Download(path.to_path_buf()));
                            plan.stats.files_to_download += 1;
                            plan.stats.bytes_to_download += remote_file.size;
                        } else {
                            // Timestamps differ but logic unsure, treat as conflict
                            plan.stats.conflicts += 1;
                            plan.actions.push(SyncAction::Conflict(FileConflict {
                                folder_id: folder_id.to_string(),
                                path: path.to_path_buf(),
                                local_metadata: (*local_file).clone(),
                                remote_metadata: (*remote_file).clone(),
                                suggested_strategy: config.conflict_strategy,
                                timestamp: SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_millis() as i64,
                            }));
                        }
                    }
                }
                None => {
                    // Local file only. Assume ADD (Upload).
                    plan.actions.push(SyncAction::Upload(path.to_path_buf()));
                    plan.stats.files_to_upload += 1;
                    plan.stats.bytes_to_upload += local_file.size;
                }
            }
        }

        // 2. Check remote files (Downloads)
        for (path, remote_file) in &remote_map {
            if !local_map.contains_key(path) {
                // Remote file only. Assume ADD (Download).
                plan.actions.push(SyncAction::Download(path.to_path_buf()));
                plan.stats.files_to_download += 1;
                plan.stats.bytes_to_download += remote_file.size;
            }
        }

        debug!(
            "Created sync plan for {}: +{}up, +{}down, {} conflicts",
            folder_id,
            plan.stats.files_to_upload,
            plan.stats.files_to_download,
            plan.stats.conflicts
        );

        plan
    }

    /// Resolve a file conflict
    pub async fn resolve_conflict(
        &mut self,
        conflict: &FileConflict,
        strategy: ConflictStrategy,
    ) -> Result<()> {
        info!(
            "Resolving conflict for {} using {:?}",
            conflict.path.display(),
            strategy
        );

        match strategy {
            ConflictStrategy::LastModifiedWins => {
                // Use most recently modified file
                if conflict.local_metadata.modified > conflict.remote_metadata.modified {
                    // TODO: Push local file to remote
                    debug!("Local file is newer, pushing to remote");
                } else {
                    // TODO: Pull remote file to local
                    debug!("Remote file is newer, pulling from remote");
                }
            }
            ConflictStrategy::KeepBoth => {
                // Rename one file with timestamp
                // TODO: Rename conflicting file
                // TODO: Pull remote file
                debug!("Keeping both files");
            }
            ConflictStrategy::Manual => {
                // TODO: Prompt user for resolution
                // For now, keep as pending
                warn!("Manual resolution required");
                return Ok(());
            }
            ConflictStrategy::SizeBased => {
                // Keep larger file
                if conflict.local_metadata.size > conflict.remote_metadata.size {
                    // TODO: Push local file to remote
                    debug!("Local file is larger, pushing to remote");
                } else {
                    // TODO: Pull remote file to local
                    debug!("Remote file is larger, pulling from remote");
                }
            }
        }

        // Remove from pending conflicts
        self.pending_conflicts
            .retain(|c| c.folder_id != conflict.folder_id || c.path != conflict.path);

        Ok(())
    }

    /// Get list of pending conflicts
    pub fn get_pending_conflicts(&self) -> &[FileConflict] {
        &self.pending_conflicts
    }

    /// Get sync folder configuration
    pub fn get_folder_config(&self, folder_id: &str) -> Option<&SyncFolder> {
        self.sync_folders.get(folder_id)
    }

    /// Get current sync index for a folder
    pub fn get_sync_index(&self, folder_id: &str) -> Option<&SyncIndex> {
        self.sync_indexes.get(folder_id)
    }
}

impl Default for FileSyncPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for FileSyncPlugin {
    fn name(&self) -> &str {
        PLUGIN_NAME
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn incoming_capabilities(&self) -> Vec<String> {
        vec![INCOMING_CAPABILITY.to_string()]
    }

    fn outgoing_capabilities(&self) -> Vec<String> {
        vec![OUTGOING_CAPABILITY.to_string()]
    }

    async fn init(&mut self, device: &Device) -> Result<()> {
        info!("Initializing FileSync plugin for device {}", device.name());
        self.device_id = Some(device.id().to_string());

        // TODO: Load sync folder configurations from database
        // TODO: Initialize file system watchers

        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        info!("Starting FileSync plugin");
        self.enabled = true;

        // Initialize watcher
        let mut watcher = RecommendedWatcher::new(
            |res: notify::Result<notify::Event>| match res {
                Ok(event) => info!("File change detected: {:?}", event),
                Err(e) => warn!("Watch error: {:?}", e),
            },
            Config::default(),
        )
        .map_err(|e| ProtocolError::Plugin(e.to_string()))?;

        // Start watching all configured folders
        for config in self.sync_folders.values() {
            if config.enabled {
                if let Err(e) = watcher.watch(&config.local_path, RecursiveMode::Recursive) {
                    warn!(
                        "Failed to watch folder {}: {}",
                        config.local_path.display(),
                        e
                    );
                }
            }
        }

        self.watcher = Some(watcher);

        // TODO: Trigger initial index generation for all folders
        // TODO: Schedule periodic scans

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        info!("Stopping FileSync plugin");
        self.enabled = false;
        self.watcher = None;

        // TODO: Cancel active transfers
        // TODO: Save sync state to database

        Ok(())
    }

    async fn handle_packet(&mut self, packet: &Packet, _device: &mut Device) -> Result<()> {
        if !self.enabled {
            debug!("FileSync plugin is disabled, ignoring packet");
            return Ok(());
        }

        debug!("Handling packet type: {}", packet.packet_type);

        match packet.packet_type.as_str() {
            "cconnect.filesync.config" => {
                // Receive sync folder configuration
                let folder_id: String = packet
                    .body
                    .get("folder_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ProtocolError::InvalidPacket("Missing folder_id".to_string()))?
                    .to_string();

                let config: SyncFolder =
                    serde_json::from_value(packet.body.get("config").cloned().ok_or_else(
                        || ProtocolError::InvalidPacket("Missing config".to_string()),
                    )?)
                    .map_err(|e| ProtocolError::InvalidPacket(e.to_string()))?;

                self.configure_folder(folder_id, config)?;

                info!("Received sync folder configuration");
            }

            "cconnect.filesync.index" => {
                // Receive remote sync index
                let index: SyncIndex = serde_json::from_value(packet.body.clone())
                    .map_err(|e| ProtocolError::InvalidPacket(e.to_string()))?;

                let folder_id = index.folder_id.clone();

                // Compare with local index
                // Compare with local index
                if let Ok(local_index) = self.generate_index(&folder_id).await {
                    let plan = self.create_sync_plan(&folder_id, &local_index, &index);

                    if plan.stats.files_to_upload > 0
                        || plan.stats.files_to_download > 0
                        || plan.stats.conflicts > 0
                    {
                        info!(
                            "Sync Plan: {} uploads, {} downloads, {} conflicts",
                            plan.stats.files_to_upload,
                            plan.stats.files_to_download,
                            plan.stats.conflicts
                        );
                    }

                    // Handle conflicts
                    for action in &plan.actions {
                        if let SyncAction::Conflict(conflict) = action {
                            self.pending_conflicts.push(conflict.clone());
                        }
                    }

                    // Store remote index
                    self.sync_indexes.insert(folder_id.clone(), index);

                    // TODO: Execute transfers (Uploads)
                    // TODO: Request downloads
                }

                info!("Processed sync index");
            }

            "cconnect.filesync.transfer" => {
                // Receive file data transfer
                let _folder_id: String = packet
                    .body
                    .get("folder_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ProtocolError::InvalidPacket("Missing folder_id".to_string()))?
                    .to_string();

                let file_path: PathBuf = packet
                    .body
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ProtocolError::InvalidPacket("Missing path".to_string()))?
                    .into();

                // TODO: Extract file data from payload
                // TODO: Write file to local filesystem
                // TODO: Update sync index
                // TODO: Apply file permissions

                debug!("Received file transfer for {}", file_path.display());
            }

            "cconnect.filesync.delete" => {
                // Synchronize file deletion
                let _folder_id: String = packet
                    .body
                    .get("folder_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ProtocolError::InvalidPacket("Missing folder_id".to_string()))?
                    .to_string();

                let file_path: PathBuf = packet
                    .body
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ProtocolError::InvalidPacket("Missing path".to_string()))?
                    .into();

                // TODO: Delete file from local filesystem
                // TODO: Update sync index
                // TODO: Archive to versioning if enabled

                info!("File deleted: {}", file_path.display());
            }

            "cconnect.filesync.conflict" => {
                // Receive conflict notification
                let conflict: FileConflict = serde_json::from_value(packet.body.clone())
                    .map_err(|e| ProtocolError::InvalidPacket(e.to_string()))?;

                self.pending_conflicts.push(conflict.clone());

                warn!(
                    "Conflict detected for {} in folder '{}'",
                    conflict.path.display(),
                    conflict.folder_id
                );
            }

            _ => {
                warn!("Unknown FileSync packet type: {}", packet.packet_type);
            }
        }

        Ok(())
    }
}

/// File Sync plugin factory
pub struct FileSyncPluginFactory;

impl PluginFactory for FileSyncPluginFactory {
    fn create(&self) -> Box<dyn Plugin> {
        Box::new(FileSyncPlugin::new())
    }

    fn name(&self) -> &str {
        PLUGIN_NAME
    }

    fn incoming_capabilities(&self) -> Vec<String> {
        vec![INCOMING_CAPABILITY.to_string()]
    }

    fn outgoing_capabilities(&self) -> Vec<String> {
        vec![OUTGOING_CAPABILITY.to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_test_device;

    #[tokio::test]
    async fn test_plugin_creation() {
        let plugin = FileSyncPlugin::new();
        assert_eq!(plugin.name(), PLUGIN_NAME);
        assert!(!plugin.enabled);
    }

    #[tokio::test]
    async fn test_configure_folder() {
        let mut plugin = FileSyncPlugin::new();
        plugin.enabled = true;

        let config = SyncFolder {
            local_path: std::env::temp_dir(),
            remote_path: PathBuf::from("/remote/path"),
            enabled: true,
            bidirectional: true,
            ignore_patterns: vec!["*.tmp".to_string()],
            conflict_strategy: ConflictStrategy::LastModifiedWins,
            versioning: true,
            version_keep: 5,
            scan_interval_secs: 60,
            bandwidth_limit_kbps: 0,
        };

        assert!(plugin
            .configure_folder("test_folder".to_string(), config)
            .is_ok());
        assert!(plugin.get_folder_config("test_folder").is_some());
    }

    #[tokio::test]
    async fn test_remove_folder() {
        let mut plugin = FileSyncPlugin::new();
        plugin.enabled = true;

        let config = SyncFolder {
            local_path: std::env::temp_dir(),
            remote_path: PathBuf::from("/remote/path"),
            enabled: true,
            bidirectional: true,
            ignore_patterns: Vec::new(),
            conflict_strategy: ConflictStrategy::default(),
            versioning: true,
            version_keep: 5,
            scan_interval_secs: 60,
            bandwidth_limit_kbps: 0,
        };

        plugin
            .configure_folder("test_folder".to_string(), config)
            .unwrap();
        assert!(plugin.remove_folder("test_folder").is_ok());
        assert!(plugin.get_folder_config("test_folder").is_none());
    }

    #[tokio::test]
    async fn test_conflict_strategies() {
        assert_eq!(
            ConflictStrategy::LastModifiedWins.as_str(),
            "last_modified_wins"
        );
        assert_eq!(ConflictStrategy::KeepBoth.as_str(), "keep_both");
        assert_eq!(ConflictStrategy::Manual.as_str(), "manual");
        assert_eq!(ConflictStrategy::SizeBased.as_str(), "size_based");
    }

    #[tokio::test]
    async fn test_sync_folder_validation() {
        let valid_config = SyncFolder {
            local_path: std::env::temp_dir(),
            remote_path: PathBuf::from("/remote/path"),
            enabled: true,
            bidirectional: true,
            ignore_patterns: Vec::new(),
            conflict_strategy: ConflictStrategy::default(),
            versioning: true,
            version_keep: 5,
            scan_interval_secs: 60,
            bandwidth_limit_kbps: 0,
        };

        assert!(valid_config.validate().is_ok());

        let invalid_config = SyncFolder {
            local_path: PathBuf::from("/nonexistent/path"),
            remote_path: PathBuf::from("/remote/path"),
            enabled: true,
            bidirectional: true,
            ignore_patterns: Vec::new(),
            conflict_strategy: ConflictStrategy::default(),
            versioning: true,
            version_keep: 5,
            scan_interval_secs: 60,
            bandwidth_limit_kbps: 0,
        };

        assert!(invalid_config.validate().is_err());
    }

    #[tokio::test]
    async fn test_plugin_initialization() {
        let device = create_test_device();
        let factory = FileSyncPluginFactory;
        let mut plugin = factory.create();

        assert!(plugin.init(&device).await.is_ok());
        assert!(plugin.start().await.is_ok());
        assert!(plugin.stop().await.is_ok());
    }

    #[tokio::test]
    async fn test_handle_config_packet() {
        let mut device = create_test_device();
        let factory = FileSyncPluginFactory;
        let mut plugin = factory.create();

        plugin.init(&device).await.unwrap();
        plugin.start().await.unwrap();

        let config = SyncFolder {
            local_path: std::env::temp_dir(),
            remote_path: PathBuf::from("/remote/path"),
            enabled: true,
            bidirectional: true,
            ignore_patterns: Vec::new(),
            conflict_strategy: ConflictStrategy::default(),
            versioning: true,
            version_keep: 5,
            scan_interval_secs: 60,
            bandwidth_limit_kbps: 0,
        };

        let mut body = serde_json::Map::new();
        body.insert(
            "folder_id".to_string(),
            serde_json::Value::String("test".to_string()),
        );
        body.insert("config".to_string(), serde_json::to_value(&config).unwrap());

        let packet = Packet::new("cconnect.filesync.config", serde_json::Value::Object(body));

        assert!(plugin.handle_packet(&packet, &mut device).await.is_ok());
    }

    #[tokio::test]
    async fn test_pending_conflicts() {
        let plugin = FileSyncPlugin::new();
        assert_eq!(plugin.get_pending_conflicts().len(), 0);
    }
}
