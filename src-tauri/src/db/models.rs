use serde::{Deserialize, Serialize};

/// meetings テーブルの行モデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meeting {
    pub id: String,
    pub title: String,
    pub recorded_at: String,
    pub audio_path: String,
    pub duration_sec: i64,
    pub status: String,
    pub created_at: String,
}

/// transcripts テーブルの行モデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub id: String,
    pub meeting_id: String,
    pub raw_text: String,
    pub edited_text: String,
    pub vosk_confidence: f64,
    pub created_at: String,
    pub updated_at: String,
}

/// minutes テーブルの行モデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Minute {
    pub id: String,
    pub meeting_id: String,
    pub section_type: String,
    pub content: String,
    pub sort_order: i64,
    pub created_at: String,
}

/// todos テーブルの行モデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub meeting_id: String,
    pub todo_text: String,
    pub due_keyword: String,
    pub is_checked: bool,
    pub is_manual: bool,
    pub is_deleted: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// summaries テーブルの行モデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub id: String,
    pub meeting_id: String,
    pub summary_text: String,
    pub created_at: String,
}

/// 会議詳細（関連データ込み）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingDetail {
    pub meeting: Meeting,
    pub transcript: Option<Transcript>,
    pub minutes: Vec<Minute>,
    pub todos: Vec<Todo>,
    pub summary: Option<Summary>,
}

/// 新規会議作成用
#[derive(Debug)]
pub struct NewMeeting {
    pub id: String,
    pub title: String,
    pub recorded_at: String,
    pub audio_path: String,
    pub duration_sec: i64,
    pub status: String,
    pub created_at: String,
}

/// 新規トランスクリプト作成用
#[derive(Debug)]
pub struct NewTranscript {
    pub id: String,
    pub meeting_id: String,
    pub raw_text: String,
    pub edited_text: String,
    pub vosk_confidence: f64,
    pub created_at: String,
    pub updated_at: String,
}

/// 新規議事録アイテム作成用
#[derive(Debug)]
pub struct NewMinute {
    pub id: String,
    pub meeting_id: String,
    pub section_type: String,
    pub content: String,
    pub sort_order: i64,
    pub created_at: String,
}

/// 新規 ToDo 作成用
#[derive(Debug)]
pub struct NewTodo {
    pub id: String,
    pub meeting_id: String,
    pub todo_text: String,
    pub due_keyword: String,
    pub is_manual: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 新規サマリー作成用
#[derive(Debug)]
pub struct NewSummary {
    pub id: String,
    pub meeting_id: String,
    pub summary_text: String,
    pub created_at: String,
}

/// フロントエンド向け議事録アイテム
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinuteItem {
    pub id: String,
    pub section_type: String,
    pub content: String,
    pub sort_order: i64,
}

/// フロントエンド向け ToDo アイテム
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub todo_text: String,
    pub due_keyword: String,
    pub is_checked: bool,
    pub is_manual: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;

    #[test]
    fn test_meeting_serialization() {
        let meeting = Meeting {
            id: "test-id".to_string(),
            title: "テスト会議".to_string(),
            recorded_at: "2026-06-02T10:00:00Z".to_string(),
            audio_path: "/tmp/test.wav".to_string(),
            duration_sec: 120,
            status: config::STATUS_DONE.to_string(),
            created_at: "2026-06-02T10:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&meeting).expect("シリアライズ失敗");
        assert!(json.contains("テスト会議"));
        assert!(json.contains("test-id"));
    }

    #[test]
    fn test_todo_bool_fields() {
        let todo = Todo {
            id: "t1".to_string(),
            meeting_id: "m1".to_string(),
            todo_text: "タスク".to_string(),
            due_keyword: "来週".to_string(),
            is_checked: false,
            is_manual: true,
            is_deleted: false,
            created_at: "2026-06-02T10:00:00Z".to_string(),
            updated_at: "2026-06-02T10:00:00Z".to_string(),
        };
        assert!(!todo.is_checked);
        assert!(todo.is_manual);
        assert!(!todo.is_deleted);
    }

    #[test]
    fn test_meeting_detail_optional_fields() {
        let detail = MeetingDetail {
            meeting: Meeting {
                id: "m1".to_string(),
                title: "テスト".to_string(),
                recorded_at: "2026-06-02T10:00:00Z".to_string(),
                audio_path: "/tmp/audio.wav".to_string(),
                duration_sec: 0,
                status: config::STATUS_DONE.to_string(),
                created_at: "2026-06-02T10:00:00Z".to_string(),
            },
            transcript: None,
            minutes: vec![],
            todos: vec![],
            summary: None,
        };
        assert!(detail.transcript.is_none());
        assert!(detail.summary.is_none());
        assert!(detail.minutes.is_empty());
        assert!(detail.todos.is_empty());
    }
}
