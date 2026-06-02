# ER図

シングルユーザー・セッション管理なし

```mermaid
erDiagram
    meetings {
        UUID id PK
        TEXT title
        DATETIME recorded_at
        TEXT audio_path
        INTEGER duration_sec
        TEXT status
        DATETIME created_at
    }

    transcripts {
        UUID id PK
        UUID meeting_id FK
        TEXT raw_text
        TEXT edited_text
        REAL vosk_confidence
        DATETIME created_at
        DATETIME updated_at
    }

    minutes {
        UUID id PK
        UUID meeting_id FK
        TEXT section_type
        TEXT content
        INTEGER sort_order
        DATETIME created_at
    }

    todos {
        UUID id PK
        UUID meeting_id FK
        TEXT todo_text
        TEXT due_keyword
        BOOLEAN is_checked
        BOOLEAN is_manual
        BOOLEAN is_deleted
        DATETIME created_at
        DATETIME updated_at
    }

    summaries {
        UUID id PK
        UUID meeting_id FK
        TEXT summary_text
        DATETIME created_at
    }

    meetings ||--o| transcripts : "has"
    meetings ||--o{ minutes : "has"
    meetings ||--o{ todos : "has"
    meetings ||--o| summaries : "has"
```

## 設計上のポイント

- Web版に存在した sessions テーブルは不要のため含まない
- 全テーブルから session_id カラムを削除
- シングルユーザーのためアクセス制御は OS 権限に委譲
- `meetings.status` の値: `recording` / `processing` / `done` / `error`
- `minutes.section_type` の値: `decisions`（決定事項）/ `next`（次回議題）/ `body`（本文）
- `todos.is_deleted` は論理削除フラグ（物理削除しない）
