# minutter

**会議録音 → 議事録・ToDo・要約 自動作成システム（デモ版）**

デスクトップアプリ。オフライン完結・外部 API 不使用・シングルユーザー。

---

## 起動方法

```bash
# 前提：Voskモデルを所定の場所に配置済みであること
# Windows: %APPDATA%\minutter\models\vosk-model-ja\
# macOS:   ~/Library/Application Support/minutter/models/vosk-model-ja/
# Linux:   ~/.local/share/minutter/models/vosk-model-ja/

npm install        # フロントエンド依存関係
cargo tauri dev    # 開発サーバー起動
```

> Vosk モデルが未配置の場合、アプリ起動時にエラー画面を表示します。自動ダウンロードは行いません。

---

## ページ（画面）一覧

| ページ名 | 遷移条件 |
|---------|---------|
| [モデルエラー画面](#) | Vosk モデル未配置時に起動直後に表示 |
| [会議一覧画面](#) | 通常起動後のホーム画面 |
| [録音・インポート画面](#) | 新規録音開始 or ファイルインポート時 |
| [文字起こし確認・編集画面](#) | 録音停止 or インポート完了後 |
| [議事録・ToDo・要約画面](#) | 生成ボタン押下後 |

---

## Tauri コマンド一覧（内部 API）

仕様書: [SPEC/API仕様書.md](SPEC/API仕様書.md)

| タイトル | コマンド |
|---------|---------|
| Vosk モデル確認 | `invoke("check_model")` |
| 録音開始 | `invoke("start_recording")` |
| 録音停止 | `invoke("stop_recording")` |
| 音声ファイルインポート | `invoke("import_audio", { path })` |
| 議事録・ToDo・要約 一括生成 | `invoke("generate_all", { meetingId })` |
| 会議一覧取得 | `invoke("list_meetings")` |
| 会議記録削除 | `invoke("delete_meeting", { id })` |

---

## 技術スタック

| レイヤー | 技術 |
|---------|------|
| フレームワーク | Tauri v2（Rust + React/TypeScript） |
| UI | React + Tailwind CSS |
| アイコン | Font Awesome |
| 音声キャプチャ | cpal |
| 音声変換 | ffmpeg-next + Tauri サイドカー |
| 音声認識 | Vosk-API（日本語モデル・オフライン） |
| テキスト処理 | ルールベース（Rust） |
| DB | SQLite（rusqlite） |
| 配布形式 | .msi / .dmg / .deb |

---

## DB ファイルパス

| OS | パス |
|----|------|
| Windows | `%APPDATA%\minutter\data.db` |
| macOS | `~/Library/Application Support/minutter/data.db` |
| Linux | `~/.local/share/minutter/data.db` |

---

## 開発者向けドキュメント

- [開発環境](ENV/DEVELOPMENT.md)
- [仕様書](SPEC/)
- [タスク](TASKS/)
- [バグ報告](DEBUG/)
