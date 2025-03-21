use std::time::{SystemTime, UNIX_EPOCH};
use rusqlite::{params, Connection, Result};
use std::fs::File;
use std::io::Write;

pub struct PlayedHistory {
    conn: Connection,
}

impl PlayedHistory {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
        // 创建播放历史表,添加wallpaper_name字段
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

        // 修改统计视图,使用最后一次播放的name
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

    // 记录壁纸开始播放,添加wallpaper_name参数
    pub fn start_playing(&self, wallpaper_id: &str, wallpaper_name: &str) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // 先处理所有未完成的记录
        self.conn.execute(
            "UPDATE play_history 
            SET end_time = start_time + 60, is_complete = 1
            WHERE end_time IS NULL",
            [],
        )?;

        // 插入新的播放记录,包含wallpaper_name
        self.conn.execute(
            "INSERT INTO play_history (wallpaper_id, wallpaper_name, start_time) 
            VALUES (?1, ?2, ?3)",
            params![wallpaper_id, wallpaper_name, now],
        )?;

        Ok(())
    }

    // 记录壁纸被切换走
    pub fn change_playing(&self, wallpaper_id: &str) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.execute(
            "UPDATE play_history 
            SET end_time = ?1, is_complete = 0
            WHERE wallpaper_id = ?2 AND end_time IS NULL",
            params![now, wallpaper_id],
        )?;

        Ok(())
    }
    // 记录壁纸完整播放结束
    pub fn complete_playing(&self, wallpaper_id: &str) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.execute(
            "UPDATE play_history 
            SET end_time = ?1, is_complete = 1
            WHERE wallpaper_id = ?2 AND end_time IS NULL",
            params![now, wallpaper_id],
        )?;

        Ok(())
    }

    // 获取壁纸的播放统计
    pub fn get_stats(&self, wallpaper_id: &str) -> Result<Option<(u32, u32, u64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT complete_plays, interrupted_plays, total_play_time 
            FROM wallpaper_stats 
            WHERE wallpaper_id = ?1"
        )?;

        let stats = stmt.query_row(
            params![wallpaper_id],
            |row| Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?
            ))
        );

        match stats {
            Ok(stats) => Ok(Some(stats)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    // 获取所有壁纸的播放统计,添加wallpaper_name
    pub fn get_all_stats(&self) -> Result<Vec<(String, String, u32, u32, u64, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT wallpaper_id, wallpaper_name, complete_plays, 
                    interrupted_plays, total_play_time, last_played 
            FROM wallpaper_stats 
            ORDER BY last_played DESC"
        )?;

        let stats = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?
            ))
        })?;

        Ok(stats.collect::<Result<Vec<_>>>()?)
    }

    // 重置所有统计数据
    pub fn reset_stats(&self) -> Result<()> {
        self.conn.execute("DELETE FROM play_history", [])?;
        Ok(())
    }

    // 导出统计数据到CSV文件
    pub fn export_stats_to_csv(&self, file_path: &str) -> Result<()> {
        let stats = self.get_all_stats()?;
        let mut file = File::create(file_path).expect(&format!("Can not create file: {}", &file_path));

        // 写入CSV头
        writeln!(file, "壁纸ID,壁纸名称,完整播放次数,中断次数,总播放时长(秒),最后播放时间");

        // 写入数据
        for (id, name, complete, interrupted, total_time, last_played) in stats {
            let last_played_time = chrono::NaiveDateTime::from_timestamp_opt(last_played, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "无效时间".to_string());

            writeln!(
                file,
                "{},{},{},{},{},{}",
                id, name, complete, interrupted, total_time, last_played_time
            );
        }

        Ok(())
    }
}
