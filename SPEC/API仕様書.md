# API仕様書（Tauri コマンド）

Tauri の `invoke()` で呼び出す内部コマンド一覧。HTTP API ではなく Rust バックエンドとの通信。

---

## check_model

Voskモデルの存在を確認する。

**呼び出し**
```ts
invoke("check_model")
```

**成功レスポンス**
```json
{ "ok": true }
```

**エラーレスポンス**
```json
{ "error": "MODEL_NOT_FOUND" }
```

---

## start_recording

マイク録音を開始する。

**呼び出し**
```ts
invoke("start_recording")
```

**成功レスポンス**
```json
{ "ok": true }
```

**エラーレスポンス**
```json
{ "error": "DEVICE_NOT_FOUND" | "ALREADY_RECORDING" }
```

---

## stop_recording

録音を停止し、WAVファイルを保存してVosk STTを実行する。

**呼び出し**
```ts
invoke("stop_recording")
```

**成功レスポンス**
```json
{
  "meeting_id": "uuid",
  "raw_text": "文字起こし結果テキスト",
  "vosk_confidence": 0.92
}
```

**エラーレスポンス**
```json
{ "error": "NOT_RECORDING" | "STT_FAILED" | "FILE_TOO_LARGE" }
```

---

## import_audio

音声ファイルをインポートし、必要に応じてWAVに変換してVosk STTを実行する。

**呼び出し**
```ts
invoke("import_audio", { path: "/path/to/file.mp3" })
```

**引数**
| 名前 | 型 | 説明 |
|------|----|------|
| path | string | インポートするファイルのパス（WAV/MP3/M4A/WebM） |

**成功レスポンス**
```json
{
  "meeting_id": "uuid",
  "raw_text": "文字起こし結果テキスト",
  "vosk_confidence": 0.88
}
```

**エラーレスポンス**
```json
{ "error": "UNSUPPORTED_FORMAT" | "FILE_TOO_LARGE" | "CONVERT_FAILED" | "STT_FAILED" }
```

---

## generate_all

文字起こしテキストから議事録・ToDo・要約を一括生成する。

**呼び出し**
```ts
invoke("generate_all", { meeting_id: "uuid" })
```

**引数**
| 名前 | 型 | 説明 |
|------|----|------|
| meeting_id | string | 対象会議の UUID |

**成功レスポンス**
```json
{
  "minutes": [
    { "section_type": "decisions", "content": "〇〇を採用することを決定", "sort_order": 0 },
    { "section_type": "next", "content": "次回は△△を検討", "sort_order": 1 },
    { "section_type": "body", "content": "議事録本文...", "sort_order": 2 }
  ],
  "todos": [
    { "id": "uuid", "todo_text": "〇〇を実装する", "due_keyword": "来週", "is_checked": false, "is_manual": false }
  ],
  "summary": "要約テキスト..."
}
```

**エラーレスポンス**
```json
{ "error": "MEETING_NOT_FOUND" | "TRANSCRIPT_NOT_FOUND" | "GENERATE_FAILED" }
```

---

## list_meetings

保存済みの会議一覧を取得する。

**呼び出し**
```ts
invoke("list_meetings")
```

**成功レスポンス**
```json
[
  {
    "id": "uuid",
    "title": "2026-06-02の会議",
    "recorded_at": "2026-06-02T10:00:00",
    "duration_sec": 3600,
    "status": "done",
    "created_at": "2026-06-02T11:00:00"
  }
]
```

---

## delete_meeting

会議記録を削除する（関連する transcript・minutes・todos・summaries も削除）。

**呼び出し**
```ts
invoke("delete_meeting", { id: "uuid" })
```

**引数**
| 名前 | 型 | 説明 |
|------|----|------|
| id | string | 削除する会議の UUID |

**成功レスポンス**
```json
{ "ok": true }
```

**エラーレスポンス**
```json
{ "error": "MEETING_NOT_FOUND" | "DELETE_FAILED" }
```
