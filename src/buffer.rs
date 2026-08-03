//! SQLite offline buffer for telemetry awaiting cloud sync.

use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use tracing::info;

#[derive(Debug)]
pub struct BufferedItem {
    pub id: i64,
    pub src_addr: u16,
    pub packet_type: u8,
    pub payload: Vec<u8>,
}

pub struct OfflineBuffer {
    conn: Connection,
}

impl OfflineBuffer {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        let conn = Connection::open(path)
            .with_context(|| format!("open sqlite {}", path.display()))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS telemetry_buffer (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                src_addr INTEGER NOT NULL,
                packet_type INTEGER NOT NULL,
                payload BLOB NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )?;
        info!(path = %path.display(), "offline buffer ready");
        Ok(Self { conn })
    }

    pub fn enqueue_telemetry(
        &mut self,
        src_addr: u16,
        packet_type: u8,
        payload: &[u8],
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO telemetry_buffer (src_addr, packet_type, payload) VALUES (?1, ?2, ?3)",
            params![src_addr as i64, packet_type as i64, payload],
        )?;
        Ok(())
    }

    pub fn drain_batch(&mut self, limit: usize) -> Result<Vec<BufferedItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, src_addr, packet_type, payload FROM telemetry_buffer
             ORDER BY id ASC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(BufferedItem {
                id: row.get(0)?,
                src_addr: row.get::<_, i64>(1)? as u16,
                packet_type: row.get::<_, i64>(2)? as u8,
                payload: row.get(3)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    pub fn delete_ids(&mut self, ids: &[i64]) -> Result<()> {
        let tx = self.conn.transaction()?;
        for id in ids {
            tx.execute("DELETE FROM telemetry_buffer WHERE id = ?1", params![id])?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn len(&self) -> Result<usize> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM telemetry_buffer", [], |row| row.get(0))?;
        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn enqueue_and_drain() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        let mut buf = OfflineBuffer::open(&path).unwrap();
        buf.enqueue_telemetry(1, 0x01, b"\x01\x02").unwrap();
        assert_eq!(buf.len().unwrap(), 1);
        let items = buf.drain_batch(10).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].payload, b"\x01\x02");
        buf.delete_ids(&[items[0].id]).unwrap();
        assert_eq!(buf.len().unwrap(), 0);
    }
}
