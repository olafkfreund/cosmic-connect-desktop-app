//! SQLite Storage Backend for Chat Plugin
//!
//! Provides persistent storage for chat messages using SQLite.
//! Messages are stored per-device with configurable retention.
//!
//! ## Database Schema
//!
//! ```sql
//! CREATE TABLE messages (
//!     id INTEGER PRIMARY KEY AUTOINCREMENT,
//!     device_id TEXT NOT NULL,
//!     message_id TEXT NOT NULL UNIQUE,
//!     text TEXT NOT NULL,
//!     timestamp INTEGER NOT NULL,
//!     from_me INTEGER NOT NULL,
//!     read INTEGER NOT NULL DEFAULT 0,
//!     created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
//! );
//!
//! CREATE INDEX idx_messages_device ON messages(device_id);
//! CREATE INDEX idx_messages_timestamp ON messages(device_id, timestamp DESC);
//! ```
//!
//! ## Storage Location
//!
//! Default path: `~/.local/share/cosmic-connect/chat.db`

use rusqlite::{params, Connection, Result as SqliteResult};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

use super::chat::{ChatConfig, ChatMessage};

/// SQLite-backed chat storage
pub struct ChatSqliteStorage {
    /// Database connection
    conn: Arc<Mutex<Connection>>,
    /// Device ID for this storage instance
    device_id: String,
    /// Configuration
    config: ChatConfig,
}

impl ChatSqliteStorage {
    /// Create new storage for a device
    ///
    /// # Arguments
    /// * `device_id` - Device ID to store messages for
    /// * `config` - Chat configuration
    ///
    /// # Returns
    /// New storage instance or error
    pub fn new(device_id: &str, config: ChatConfig) -> Result<Self, String> {
        let db_path = Self::get_db_path()?;
        Self::new_with_path(device_id, config, &db_path)
    }

    /// Create storage with explicit database path (for testing)
    pub fn new_with_path(
        device_id: &str,
        config: ChatConfig,
        db_path: &PathBuf,
    ) -> Result<Self, String> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create db directory: {}", e))?;
        }

        let conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
            device_id: device_id.to_string(),
            config,
        };

        storage.init_schema()?;
        Ok(storage)
    }

    /// Get the default database path
    fn get_db_path() -> Result<PathBuf, String> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| "Could not determine local data directory".to_string())?;
        Ok(data_dir.join("cosmic-connect").join("chat.db"))
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                device_id TEXT NOT NULL,
                message_id TEXT NOT NULL,
                text TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                from_me INTEGER NOT NULL,
                read INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                UNIQUE(device_id, message_id)
            );

            CREATE INDEX IF NOT EXISTS idx_messages_device
                ON messages(device_id);
            CREATE INDEX IF NOT EXISTS idx_messages_timestamp
                ON messages(device_id, timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_messages_unread
                ON messages(device_id, read) WHERE read = 0;
            "#,
        )
        .map_err(|e| format!("Failed to create schema: {}", e))?;

        debug!("Chat database schema initialized");
        Ok(())
    }

    /// Add a message to storage
    pub fn add(&self, message: &ChatMessage) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        conn.execute(
            r#"
            INSERT OR REPLACE INTO messages
                (device_id, message_id, text, timestamp, from_me, read)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                self.device_id,
                message.message_id,
                message.text,
                message.timestamp,
                message.from_me as i32,
                message.read as i32,
            ],
        )
        .map_err(|e| format!("Failed to insert message: {}", e))?;

        debug!("Added message {} for device {}", message.message_id, self.device_id);

        // Cleanup old messages
        drop(conn);
        self.cleanup()?;

        Ok(())
    }

    /// Get a message by ID
    pub fn get(&self, message_id: &str) -> Result<Option<ChatMessage>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare(
                r#"
                SELECT message_id, text, timestamp, from_me, read
                FROM messages
                WHERE device_id = ?1 AND message_id = ?2
                "#,
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        stmt.query_row(params![self.device_id, message_id], row_to_message)
            .optional()
            .map_err(|e| format!("Failed to query message: {}", e))
    }

    /// Mark a message as read
    pub fn mark_read(&self, message_id: &str) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let rows = conn
            .execute(
                r#"
                UPDATE messages
                SET read = 1
                WHERE device_id = ?1 AND message_id = ?2
                "#,
                params![self.device_id, message_id],
            )
            .map_err(|e| format!("Failed to update message: {}", e))?;

        if rows > 0 {
            debug!("Marked message {} as read", message_id);
            Ok(true)
        } else {
            warn!("Message {} not found for device {}", message_id, self.device_id);
            Ok(false)
        }
    }

    /// Get message history
    ///
    /// # Arguments
    /// * `limit` - Maximum messages to return
    /// * `before_timestamp` - Only return messages before this timestamp (optional)
    ///
    /// # Returns
    /// Vector of messages, newest first
    pub fn get_history(
        &self,
        limit: usize,
        before_timestamp: Option<i64>,
    ) -> Result<Vec<ChatMessage>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let messages: Vec<ChatMessage> = if let Some(before) = before_timestamp {
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT message_id, text, timestamp, from_me, read
                    FROM messages
                    WHERE device_id = ?1 AND timestamp < ?2
                    ORDER BY timestamp DESC
                    LIMIT ?3
                    "#,
                )
                .map_err(|e| format!("Failed to prepare query: {}", e))?;

            let rows = stmt
                .query_map(params![self.device_id, before, limit as i64], row_to_message)
                .map_err(|e| format!("Failed to query messages: {}", e))?;
            rows.filter_map(Result::ok).collect()
        } else {
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT message_id, text, timestamp, from_me, read
                    FROM messages
                    WHERE device_id = ?1
                    ORDER BY timestamp DESC
                    LIMIT ?2
                    "#,
                )
                .map_err(|e| format!("Failed to prepare query: {}", e))?;

            let rows = stmt
                .query_map(params![self.device_id, limit as i64], row_to_message)
                .map_err(|e| format!("Failed to query messages: {}", e))?;
            rows.filter_map(Result::ok).collect()
        };

        debug!(
            "Retrieved {} messages for device {}",
            messages.len(),
            self.device_id
        );

        Ok(messages)
    }

    /// Get unread message count
    pub fn unread_count(&self) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let count: i64 = conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM messages
                WHERE device_id = ?1 AND read = 0 AND from_me = 0
                "#,
                params![self.device_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to count unread: {}", e))?;

        Ok(count as usize)
    }

    /// Cleanup old messages based on retention policy
    pub fn cleanup(&self) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let cutoff_time = chrono::Utc::now().timestamp_millis()
            - (self.config.retention_days * 24 * 60 * 60 * 1000);

        // Delete old messages
        let deleted_old = conn
            .execute(
                r#"
                DELETE FROM messages
                WHERE device_id = ?1 AND timestamp < ?2
                "#,
                params![self.device_id, cutoff_time],
            )
            .map_err(|e| format!("Failed to delete old messages: {}", e))?;

        // Limit total count per device
        let deleted_excess = conn
            .execute(
                r#"
                DELETE FROM messages
                WHERE device_id = ?1 AND id NOT IN (
                    SELECT id FROM messages
                    WHERE device_id = ?1
                    ORDER BY timestamp DESC
                    LIMIT ?2
                )
                "#,
                params![self.device_id, self.config.max_messages as i64],
            )
            .map_err(|e| format!("Failed to limit messages: {}", e))?;

        let total_deleted = deleted_old + deleted_excess;
        if total_deleted > 0 {
            info!(
                "Cleaned up {} messages for device {} ({} old, {} excess)",
                total_deleted, self.device_id, deleted_old, deleted_excess
            );
        }

        Ok(total_deleted)
    }

    /// Delete all messages for this device
    pub fn clear(&self) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let deleted = conn
            .execute(
                "DELETE FROM messages WHERE device_id = ?1",
                params![self.device_id],
            )
            .map_err(|e| format!("Failed to clear messages: {}", e))?;

        info!("Cleared {} messages for device {}", deleted, self.device_id);
        Ok(deleted)
    }
}

/// Extension trait for rusqlite to support optional results
trait OptionalExt<T> {
    fn optional(self) -> SqliteResult<Option<T>>;
}

impl<T> OptionalExt<T> for SqliteResult<T> {
    fn optional(self) -> SqliteResult<Option<T>> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

/// Map a database row to a ChatMessage
fn row_to_message(row: &rusqlite::Row) -> rusqlite::Result<ChatMessage> {
    Ok(ChatMessage {
        message_id: row.get(0)?,
        text: row.get(1)?,
        timestamp: row.get(2)?,
        from_me: row.get::<_, i32>(3)? != 0,
        read: row.get::<_, i32>(4)? != 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_storage() -> (ChatSqliteStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_chat.db");
        let config = ChatConfig::default();
        let storage = ChatSqliteStorage::new_with_path("test_device", config, &db_path).unwrap();
        (storage, temp_dir)
    }

    #[test]
    fn test_add_and_get_message() {
        let (storage, _temp) = create_test_storage();

        let msg = ChatMessage::new("Hello, world!".to_string(), true);
        let msg_id = msg.message_id.clone();

        storage.add(&msg).unwrap();

        let retrieved = storage.get(&msg_id).unwrap().unwrap();
        assert_eq!(retrieved.message_id, msg_id);
        assert_eq!(retrieved.text, "Hello, world!");
        assert!(retrieved.from_me);
        assert!(!retrieved.read);
    }

    #[test]
    fn test_mark_read() {
        let (storage, _temp) = create_test_storage();

        let msg = ChatMessage::new("Test message".to_string(), false);
        let msg_id = msg.message_id.clone();

        storage.add(&msg).unwrap();
        assert!(!storage.get(&msg_id).unwrap().unwrap().read);

        storage.mark_read(&msg_id).unwrap();
        assert!(storage.get(&msg_id).unwrap().unwrap().read);
    }

    #[test]
    fn test_get_history() {
        let (storage, _temp) = create_test_storage();

        // Use realistic timestamps based on current time to avoid cleanup deletion
        let base_time = chrono::Utc::now().timestamp_millis();

        // Add some messages
        for i in 0..5 {
            let mut msg = ChatMessage::new(format!("Message {}", i), i % 2 == 0);
            msg.timestamp = base_time + i as i64 * 1000; // 1 second apart
            storage.add(&msg).unwrap();
        }

        let history = storage.get_history(10, None).unwrap();
        assert_eq!(history.len(), 5);

        // Should be newest first
        assert!(history[0].timestamp > history[4].timestamp);
    }

    #[test]
    fn test_get_history_with_before() {
        let (storage, _temp) = create_test_storage();

        // Use realistic timestamps based on current time
        let base_time = chrono::Utc::now().timestamp_millis();

        for i in 0..5 {
            let mut msg = ChatMessage::new(format!("Message {}", i), true);
            msg.timestamp = base_time + i as i64 * 1000; // 1 second apart
            storage.add(&msg).unwrap();
        }

        // Get messages before the 3rd message (should exclude last 2)
        let cutoff = base_time + 3 * 1000;
        let history = storage.get_history(10, Some(cutoff)).unwrap();
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn test_unread_count() {
        let (storage, _temp) = create_test_storage();

        // Add received (not from_me) messages
        for i in 0..3 {
            let msg = ChatMessage::new(format!("Received {}", i), false);
            storage.add(&msg).unwrap();
        }

        // Add sent messages (from_me)
        for i in 0..2 {
            let msg = ChatMessage::new(format!("Sent {}", i), true);
            storage.add(&msg).unwrap();
        }

        // Only count received unread messages
        assert_eq!(storage.unread_count().unwrap(), 3);
    }

    #[test]
    fn test_clear() {
        let (storage, _temp) = create_test_storage();

        for i in 0..5 {
            let msg = ChatMessage::new(format!("Message {}", i), true);
            storage.add(&msg).unwrap();
        }

        assert_eq!(storage.get_history(100, None).unwrap().len(), 5);

        storage.clear().unwrap();
        assert_eq!(storage.get_history(100, None).unwrap().len(), 0);
    }
}
