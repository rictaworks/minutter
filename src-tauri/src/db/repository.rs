use rusqlite::{Connection, params};
use log::{debug, info, error};
use crate::config;
use crate::error::AppError;
use crate::db::models::{
    Meeting, MeetingDetail, Minute, MinuteItem, NewMeeting, NewMinute,
    NewSummary, NewTodo, NewTranscript, Summary, Todo, TodoItem, Transcript,
};

/// SQLite に対する全 CRUD 操作を担当するリポジトリ
pub struct MeetingRepository {
    conn: Connection,
}

impl MeetingRepository {
    /// 指定したパスで接続を作成し DB を初期化する
    pub fn new(db_path: &str) -> Result<Self, AppError> {
        debug!("DB 接続開始: {}", db_path);
        let conn = Connection::open(db_path).map_err(|e| {
            error!("DB 接続失敗: {}", e);
            AppError::DbInitFailed(e.to_string())
        })?;
        let repo = MeetingRepository { conn };
        repo.init_db()?;
        Ok(repo)
    }

    /// インメモリ DB で接続を作成する（テスト用）
    pub fn new_in_memory() -> Result<Self, AppError> {
        debug!("インメモリ DB 接続開始");
        let conn = Connection::open_in_memory().map_err(|e| {
            error!("インメモリ DB 接続失敗: {}", e);
            AppError::DbInitFailed(e.to_string())
        })?;
        let repo = MeetingRepository { conn };
        repo.init_db()?;
        Ok(repo)
    }

    /// テーブルを初期化する
    pub fn init_db(&self) -> Result<(), AppError> {
        info!("DB 初期化開始");
        self.conn.execute_batch(config::SQL_PRAGMA_FOREIGN_KEYS).map_err(|e| {
            error!("PRAGMA 設定失敗: {}", e);
            AppError::DbInitFailed(e.to_string())
        })?;
        self.conn.execute_batch(config::SQL_CREATE_MEETINGS).map_err(|e| {
            error!("meetings テーブル作成失敗: {}", e);
            AppError::DbInitFailed(e.to_string())
        })?;
        self.conn.execute_batch(config::SQL_CREATE_TRANSCRIPTS).map_err(|e| {
            error!("transcripts テーブル作成失敗: {}", e);
            AppError::DbInitFailed(e.to_string())
        })?;
        self.conn.execute_batch(config::SQL_CREATE_MINUTES).map_err(|e| {
            error!("minutes テーブル作成失敗: {}", e);
            AppError::DbInitFailed(e.to_string())
        })?;
        self.conn.execute_batch(config::SQL_CREATE_TODOS).map_err(|e| {
            error!("todos テーブル作成失敗: {}", e);
            AppError::DbInitFailed(e.to_string())
        })?;
        self.conn.execute_batch(config::SQL_CREATE_SUMMARIES).map_err(|e| {
            error!("summaries テーブル作成失敗: {}", e);
            AppError::DbInitFailed(e.to_string())
        })?;
        info!("DB 初期化完了");
        Ok(())
    }

    /// DB の整合性チェック
    pub fn check_integrity(&self) -> bool {
        debug!("DB 整合性チェック開始");
        let result: Result<String, _> = self.conn.query_row(
            config::SQL_PRAGMA_INTEGRITY_CHECK,
            [],
            |row| row.get(0),
        );
        match result {
            Ok(val) => {
                let ok = val == config::SQL_INTEGRITY_OK;
                if ok {
                    info!("DB 整合性チェック OK");
                } else {
                    error!("DB 整合性チェック失敗: {}", val);
                }
                ok
            }
            Err(e) => {
                error!("DB 整合性チェック エラー: {}", e);
                false
            }
        }
    }

    /// 新規会議を保存する
    pub fn save_meeting(&self, m: &NewMeeting) -> Result<(), AppError> {
        debug!("会議保存: id={}", m.id);
        self.conn.execute(
            "INSERT INTO meetings (id, title, recorded_at, audio_path, duration_sec, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![m.id, m.title, m.recorded_at, m.audio_path, m.duration_sec, m.status, m.created_at],
        ).map_err(|e| {
            error!("会議保存失敗: {}", e);
            AppError::Db(e)
        })?;
        Ok(())
    }

    /// トランスクリプトを保存する
    pub fn save_transcript(&self, t: &NewTranscript) -> Result<(), AppError> {
        debug!("トランスクリプト保存: meeting_id={}", t.meeting_id);
        self.conn.execute(
            "INSERT OR REPLACE INTO transcripts
             (id, meeting_id, raw_text, edited_text, vosk_confidence, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                t.id, t.meeting_id, t.raw_text, t.edited_text,
                t.vosk_confidence, t.created_at, t.updated_at
            ],
        ).map_err(|e| {
            error!("トランスクリプト保存失敗: {}", e);
            AppError::Db(e)
        })?;
        Ok(())
    }

    /// 議事録アイテムを一括保存する
    pub fn save_minutes(&self, items: &[NewMinute]) -> Result<(), AppError> {
        debug!("議事録保存: {} 件", items.len());
        for item in items {
            self.conn.execute(
                "INSERT INTO minutes (id, meeting_id, section_type, content, sort_order, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    item.id, item.meeting_id, item.section_type,
                    item.content, item.sort_order, item.created_at
                ],
            ).map_err(|e| {
                error!("議事録保存失敗: id={}, {}", item.id, e);
                AppError::Db(e)
            })?;
        }
        Ok(())
    }

    /// ToDo を一括保存する
    pub fn save_todos(&self, items: &[NewTodo]) -> Result<(), AppError> {
        debug!("ToDo 保存: {} 件", items.len());
        for item in items {
            self.conn.execute(
                "INSERT INTO todos
                 (id, meeting_id, todo_text, due_keyword, is_checked, is_manual, is_deleted, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 0, ?5, 0, ?6, ?7)",
                params![
                    item.id, item.meeting_id, item.todo_text, item.due_keyword,
                    item.is_manual as i32, item.created_at, item.updated_at
                ],
            ).map_err(|e| {
                error!("ToDo 保存失敗: id={}, {}", item.id, e);
                AppError::Db(e)
            })?;
        }
        Ok(())
    }

    /// サマリーを保存する
    pub fn save_summary(&self, s: &NewSummary) -> Result<(), AppError> {
        debug!("サマリー保存: meeting_id={}", s.meeting_id);
        self.conn.execute(
            "INSERT OR REPLACE INTO summaries (id, meeting_id, summary_text, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![s.id, s.meeting_id, s.summary_text, s.created_at],
        ).map_err(|e| {
            error!("サマリー保存失敗: {}", e);
            AppError::Db(e)
        })?;
        Ok(())
    }

    /// 会議一覧を取得する（論理削除済み除く）
    pub fn list_meetings(&self) -> Result<Vec<Meeting>, AppError> {
        debug!("会議一覧取得");
        let mut stmt = self.conn.prepare(
            "SELECT id, title, recorded_at, audio_path, duration_sec, status, created_at
             FROM meetings ORDER BY created_at DESC"
        ).map_err(AppError::Db)?;

        let meetings = stmt.query_map([], |row| {
            Ok(Meeting {
                id: row.get(0)?,
                title: row.get(1)?,
                recorded_at: row.get(2)?,
                audio_path: row.get(3)?,
                duration_sec: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
            })
        }).map_err(AppError::Db)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::Db)?;

        debug!("会議一覧取得完了: {} 件", meetings.len());
        Ok(meetings)
    }

    /// 特定の会議の詳細を取得する
    pub fn get_meeting(&self, id: &str) -> Result<Option<MeetingDetail>, AppError> {
        debug!("会議詳細取得: id={}", id);

        let meeting_opt: Option<Meeting> = {
            let mut stmt = self.conn.prepare(
                "SELECT id, title, recorded_at, audio_path, duration_sec, status, created_at
                 FROM meetings WHERE id = ?1"
            ).map_err(AppError::Db)?;
            let mut rows = stmt.query_map(params![id], |row| {
                Ok(Meeting {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    recorded_at: row.get(2)?,
                    audio_path: row.get(3)?,
                    duration_sec: row.get(4)?,
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                })
            }).map_err(AppError::Db)?;
            rows.next().transpose().map_err(AppError::Db)?
        };

        let meeting = match meeting_opt {
            None => {
                debug!("会議が見つからない: id={}", id);
                return Ok(None);
            }
            Some(m) => m,
        };

        let transcript = self.get_transcript(id)?;
        let minutes = self.get_minutes(id)?;
        let todos = self.get_todos(id)?;
        let summary = self.get_summary(id)?;

        Ok(Some(MeetingDetail {
            meeting,
            transcript,
            minutes,
            todos,
            summary,
        }))
    }

    /// 会議のトランスクリプトを取得する
    fn get_transcript(&self, meeting_id: &str) -> Result<Option<Transcript>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, meeting_id, raw_text, edited_text, vosk_confidence, created_at, updated_at
             FROM transcripts WHERE meeting_id = ?1"
        ).map_err(AppError::Db)?;
        let mut rows = stmt.query_map(params![meeting_id], |row| {
            Ok(Transcript {
                id: row.get(0)?,
                meeting_id: row.get(1)?,
                raw_text: row.get(2)?,
                edited_text: row.get(3)?,
                vosk_confidence: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        }).map_err(AppError::Db)?;
        rows.next().transpose().map_err(AppError::Db)
    }

    /// 会議の議事録アイテムを取得する
    fn get_minutes(&self, meeting_id: &str) -> Result<Vec<Minute>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, meeting_id, section_type, content, sort_order, created_at
             FROM minutes WHERE meeting_id = ?1 ORDER BY sort_order ASC"
        ).map_err(AppError::Db)?;
        let items = stmt.query_map(params![meeting_id], |row| {
            Ok(Minute {
                id: row.get(0)?,
                meeting_id: row.get(1)?,
                section_type: row.get(2)?,
                content: row.get(3)?,
                sort_order: row.get(4)?,
                created_at: row.get(5)?,
            })
        }).map_err(AppError::Db)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::Db)?;
        Ok(items)
    }

    /// 会議の ToDo を取得する（論理削除済み除く）
    fn get_todos(&self, meeting_id: &str) -> Result<Vec<Todo>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, meeting_id, todo_text, due_keyword, is_checked, is_manual, is_deleted, created_at, updated_at
             FROM todos WHERE meeting_id = ?1 AND is_deleted = 0 ORDER BY created_at ASC"
        ).map_err(AppError::Db)?;
        let items = stmt.query_map(params![meeting_id], |row| {
            Ok(Todo {
                id: row.get(0)?,
                meeting_id: row.get(1)?,
                todo_text: row.get(2)?,
                due_keyword: row.get(3)?,
                is_checked: row.get::<_, i32>(4)? != 0,
                is_manual: row.get::<_, i32>(5)? != 0,
                is_deleted: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        }).map_err(AppError::Db)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::Db)?;
        Ok(items)
    }

    /// 会議のサマリーを取得する
    fn get_summary(&self, meeting_id: &str) -> Result<Option<Summary>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, meeting_id, summary_text, created_at
             FROM summaries WHERE meeting_id = ?1"
        ).map_err(AppError::Db)?;
        let mut rows = stmt.query_map(params![meeting_id], |row| {
            Ok(Summary {
                id: row.get(0)?,
                meeting_id: row.get(1)?,
                summary_text: row.get(2)?,
                created_at: row.get(3)?,
            })
        }).map_err(AppError::Db)?;
        rows.next().transpose().map_err(AppError::Db)
    }

    /// 会議を論理削除する（関連データも削除）
    pub fn delete_meeting(&self, id: &str) -> Result<(), AppError> {
        debug!("会議削除: id={}", id);
        self.conn.execute("DELETE FROM summaries WHERE meeting_id = ?1", params![id])
            .map_err(AppError::Db)?;
        self.conn.execute("DELETE FROM todos WHERE meeting_id = ?1", params![id])
            .map_err(AppError::Db)?;
        self.conn.execute("DELETE FROM minutes WHERE meeting_id = ?1", params![id])
            .map_err(AppError::Db)?;
        self.conn.execute("DELETE FROM transcripts WHERE meeting_id = ?1", params![id])
            .map_err(AppError::Db)?;
        self.conn.execute("DELETE FROM meetings WHERE id = ?1", params![id])
            .map_err(AppError::Db)?;
        info!("会議削除完了: id={}", id);
        Ok(())
    }

    /// トランスクリプトの編集テキストを更新する
    pub fn update_transcript(&self, meeting_id: &str, edited_text: &str) -> Result<(), AppError> {
        debug!("トランスクリプト更新: meeting_id={}", meeting_id);
        let now = chrono::Utc::now().to_rfc3339();
        let affected = self.conn.execute(
            "UPDATE transcripts SET edited_text = ?1, updated_at = ?2 WHERE meeting_id = ?3",
            params![edited_text, now, meeting_id],
        ).map_err(|e| {
            error!("トランスクリプト更新失敗: {}", e);
            AppError::Db(e)
        })?;
        if affected == 0 {
            return Err(AppError::Internal(format!(
                "トランスクリプトが見つかりません: meeting_id={}", meeting_id
            )));
        }
        Ok(())
    }

    /// ToDo のチェック状態を更新する
    pub fn update_todo(&self, id: &str, is_checked: bool) -> Result<(), AppError> {
        debug!("ToDo チェック更新: id={}, is_checked={}", id, is_checked);
        let now = chrono::Utc::now().to_rfc3339();
        let affected = self.conn.execute(
            "UPDATE todos SET is_checked = ?1, updated_at = ?2 WHERE id = ?3 AND is_deleted = 0",
            params![is_checked as i32, now, id],
        ).map_err(|e| {
            error!("ToDo チェック更新失敗: {}", e);
            AppError::Db(e)
        })?;
        if affected == 0 {
            return Err(AppError::Internal(format!("ToDo が見つかりません: id={}", id)));
        }
        Ok(())
    }

    /// ToDo を論理削除する
    pub fn delete_todo(&self, id: &str) -> Result<(), AppError> {
        debug!("ToDo 論理削除: id={}", id);
        let now = chrono::Utc::now().to_rfc3339();
        let affected = self.conn.execute(
            "UPDATE todos SET is_deleted = 1, updated_at = ?1 WHERE id = ?2",
            params![now, id],
        ).map_err(|e| {
            error!("ToDo 論理削除失敗: {}", e);
            AppError::Db(e)
        })?;
        if affected == 0 {
            return Err(AppError::Internal(format!("ToDo が見つかりません: id={}", id)));
        }
        Ok(())
    }

    /// 会議のステータスを更新する
    pub fn update_meeting_status(&self, id: &str, status: &str) -> Result<(), AppError> {
        debug!("会議ステータス更新: id={}, status={}", id, status);
        let affected = self.conn.execute(
            "UPDATE meetings SET status = ?1 WHERE id = ?2",
            params![status, id],
        ).map_err(|e| {
            error!("会議ステータス更新失敗: {}", e);
            AppError::Db(e)
        })?;
        if affected == 0 {
            return Err(AppError::Internal(format!("会議が見つかりません: id={}", id)));
        }
        Ok(())
    }

    /// 会議の音声パスを更新する
    pub fn update_meeting_audio_path(&self, id: &str, audio_path: &str) -> Result<(), AppError> {
        debug!("会議音声パス更新: id={}", id);
        self.conn.execute(
            "UPDATE meetings SET audio_path = ?1 WHERE id = ?2",
            params![audio_path, id],
        ).map_err(|e| {
            error!("会議音声パス更新失敗: {}", e);
            AppError::Db(e)
        })?;
        Ok(())
    }

    /// 会議の録音時間を更新する
    pub fn update_meeting_duration(&self, id: &str, duration_sec: i64) -> Result<(), AppError> {
        debug!("会議録音時間更新: id={}, duration_sec={}", id, duration_sec);
        self.conn.execute(
            "UPDATE meetings SET duration_sec = ?1 WHERE id = ?2",
            params![duration_sec, id],
        ).map_err(|e| {
            error!("会議録音時間更新失敗: {}", e);
            AppError::Db(e)
        })?;
        Ok(())
    }

    /// minutes テーブルから MinuteItem リストを取得する（フロントエンド向け）
    pub fn get_minute_items(&self, meeting_id: &str) -> Result<Vec<MinuteItem>, AppError> {
        let minutes = self.get_minutes(meeting_id)?;
        Ok(minutes.into_iter().map(|m| MinuteItem {
            id: m.id,
            section_type: m.section_type,
            content: m.content,
            sort_order: m.sort_order,
        }).collect())
    }

    /// todos テーブルから TodoItem リストを取得する（フロントエンド向け）
    pub fn get_todo_items(&self, meeting_id: &str) -> Result<Vec<TodoItem>, AppError> {
        let todos = self.get_todos(meeting_id)?;
        Ok(todos.into_iter().map(|t| TodoItem {
            id: t.id,
            todo_text: t.todo_text,
            due_keyword: t.due_keyword,
            is_checked: t.is_checked,
            is_manual: t.is_manual,
        }).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn now_str() -> String {
        chrono::Utc::now().to_rfc3339()
    }

    fn make_meeting(id: &str, title: &str) -> NewMeeting {
        NewMeeting {
            id: id.to_string(),
            title: title.to_string(),
            recorded_at: now_str(),
            audio_path: "/tmp/test.wav".to_string(),
            duration_sec: 60,
            status: config::STATUS_DONE.to_string(),
            created_at: now_str(),
        }
    }

    #[test]
    fn test_init_db_success() {
        let repo = MeetingRepository::new_in_memory();
        assert!(repo.is_ok(), "インメモリ DB 初期化失敗: {:?}", repo.err());
    }

    #[test]
    fn test_check_integrity() {
        let repo = MeetingRepository::new_in_memory().expect("DB 初期化失敗");
        assert!(repo.check_integrity(), "整合性チェック失敗");
    }

    #[test]
    fn test_save_and_list_meetings() {
        let repo = MeetingRepository::new_in_memory().expect("DB 初期化失敗");
        let id = Uuid::new_v4().to_string();
        repo.save_meeting(&make_meeting(&id, "テスト会議")).expect("保存失敗");
        let list = repo.list_meetings().expect("一覧取得失敗");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title, "テスト会議");
    }

    #[test]
    fn test_save_transcript() {
        let repo = MeetingRepository::new_in_memory().expect("DB 初期化失敗");
        let meeting_id = Uuid::new_v4().to_string();
        repo.save_meeting(&make_meeting(&meeting_id, "テスト")).expect("保存失敗");

        let transcript = NewTranscript {
            id: Uuid::new_v4().to_string(),
            meeting_id: meeting_id.clone(),
            raw_text: "テスト音声テキスト".to_string(),
            edited_text: "".to_string(),
            vosk_confidence: 0.85,
            created_at: now_str(),
            updated_at: now_str(),
        };
        repo.save_transcript(&transcript).expect("トランスクリプト保存失敗");

        let detail = repo.get_meeting(&meeting_id).expect("取得失敗").expect("存在しない");
        assert!(detail.transcript.is_some());
        assert_eq!(detail.transcript.unwrap().raw_text, "テスト音声テキスト");
    }

    #[test]
    fn test_save_and_get_todos() {
        let repo = MeetingRepository::new_in_memory().expect("DB 初期化失敗");
        let meeting_id = Uuid::new_v4().to_string();
        repo.save_meeting(&make_meeting(&meeting_id, "テスト")).expect("保存失敗");

        let todos = vec![
            NewTodo {
                id: Uuid::new_v4().to_string(),
                meeting_id: meeting_id.clone(),
                todo_text: "来週中に資料を作成する".to_string(),
                due_keyword: "来週".to_string(),
                is_manual: false,
                created_at: now_str(),
                updated_at: now_str(),
            },
        ];
        repo.save_todos(&todos).expect("ToDo 保存失敗");

        let detail = repo.get_meeting(&meeting_id).expect("取得失敗").expect("存在しない");
        assert_eq!(detail.todos.len(), 1);
        assert_eq!(detail.todos[0].due_keyword, "来週");
    }

    #[test]
    fn test_delete_todo_logical() {
        let repo = MeetingRepository::new_in_memory().expect("DB 初期化失敗");
        let meeting_id = Uuid::new_v4().to_string();
        repo.save_meeting(&make_meeting(&meeting_id, "テスト")).expect("保存失敗");

        let todo_id = Uuid::new_v4().to_string();
        repo.save_todos(&[NewTodo {
            id: todo_id.clone(),
            meeting_id: meeting_id.clone(),
            todo_text: "確認する".to_string(),
            due_keyword: "".to_string(),
            is_manual: true,
            created_at: now_str(),
            updated_at: now_str(),
        }]).expect("ToDo 保存失敗");

        repo.delete_todo(&todo_id).expect("論理削除失敗");

        let detail = repo.get_meeting(&meeting_id).expect("取得失敗").expect("存在しない");
        // 論理削除済みなので一覧に出ない
        assert_eq!(detail.todos.len(), 0);
    }

    #[test]
    fn test_update_meeting_status() {
        let repo = MeetingRepository::new_in_memory().expect("DB 初期化失敗");
        let meeting_id = Uuid::new_v4().to_string();
        repo.save_meeting(&make_meeting(&meeting_id, "テスト")).expect("保存失敗");
        repo.update_meeting_status(&meeting_id, config::STATUS_PROCESSING).expect("ステータス更新失敗");

        let list = repo.list_meetings().expect("一覧取得失敗");
        assert_eq!(list[0].status, config::STATUS_PROCESSING);
    }

    #[test]
    fn test_delete_meeting_cascades() {
        let repo = MeetingRepository::new_in_memory().expect("DB 初期化失敗");
        let meeting_id = Uuid::new_v4().to_string();
        repo.save_meeting(&make_meeting(&meeting_id, "テスト")).expect("保存失敗");
        repo.save_todos(&[NewTodo {
            id: Uuid::new_v4().to_string(),
            meeting_id: meeting_id.clone(),
            todo_text: "タスク".to_string(),
            due_keyword: "".to_string(),
            is_manual: false,
            created_at: now_str(),
            updated_at: now_str(),
        }]).expect("ToDo 保存失敗");

        repo.delete_meeting(&meeting_id).expect("削除失敗");
        let list = repo.list_meetings().expect("一覧取得失敗");
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_get_meeting_not_found() {
        let repo = MeetingRepository::new_in_memory().expect("DB 初期化失敗");
        let result = repo.get_meeting("non-existent-id").expect("クエリ失敗");
        assert!(result.is_none());
    }
}
