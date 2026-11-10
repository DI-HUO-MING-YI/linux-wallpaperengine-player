use std::fs::File;
use std::io::Write;

use anyhow::Result;
use rusqlite::Connection;

pub struct PlayedHistory {
    conn: Connection,
}

impl PlayedHistory {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS play_history (
                id INTEGER PRIMARY KEY,
                wallpaper_id TEXT NOT NULL,
                wallpaper_name TEXT NOT NULL,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                is_complete INTEGER NOT NULL DEFAULT 1,
                CHECK (end_time IS NULL OR end_time >= start_time)
            )",
            [],
        )?;

        conn.execute(
            "CREATE VIEW IF NOT EXISTS wallpaper_stats AS
            WITH latest_play AS (
                SELECT wallpaper_id, wallpaper_name, start_time
                FROM play_history ph1
                WHERE start_time = (
                    SELECT MAX(start_time)
                    FROM play_history ph2
                    WHERE ph2.wallpaper_id = ph1.wallpaper_id
                )
            )
            SELECT
                ph.wallpaper_id,
                lp.wallpaper_name,
                COUNT(CASE WHEN ph.is_complete = 1 THEN 1 END) as complete_plays,
                COUNT(CASE WHEN ph.is_complete = 0 AND ph.end_time IS NOT NULL THEN 1 END) as interrupted_plays,
                SUM(CASE
                    WHEN ph.end_time IS NOT NULL THEN (ph.end_time - ph.start_time)
                    ELSE (ph.start_time + 60 - ph.start_time)
                END) as total_play_time,
                MAX(ph.start_time) as last_played
            FROM play_history ph
            JOIN latest_play lp ON ph.wallpaper_id = lp.wallpaper_id
            GROUP BY ph.wallpaper_id",
            [],
        )?;

        Ok(PlayedHistory { conn })
    }

    pub fn get_all_stats(&self) -> Result<Vec<(String, String, u32, u32, u64, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT wallpaper_id, wallpaper_name, complete_plays,
                    interrupted_plays, total_play_time, last_played
            FROM wallpaper_stats
            ORDER BY last_played DESC",
        )?;

        let stats = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?;

        Ok(stats.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn export_stats_to_csv(&self, file_path: &str) -> Result<()> {
        let stats = self.get_all_stats()?;
        let mut file = File::create(file_path)?;

        writeln!(
            file,
            "壁纸ID,壁纸名称,完整播放次数,中断次数,总播放时长(秒),最后播放时间"
        )?;

        for (id, name, complete, interrupted, total_time, last_played) in stats {
            let last_played_time = chrono::DateTime::from_timestamp(last_played, 0)
                .map(|date_time| date_time.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "无效时间".to_string());

            writeln!(
                file,
                "{},{},{},{},{},{}",
                id, name, complete, interrupted, total_time, last_played_time
            )?;
        }

        Ok(())
    }
}
