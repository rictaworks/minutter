# Claude Safety Rules

## 削除系コマンドの禁止（重要）

以下のルールはこのワークスペース内のすべての会話で絶対に守られる：

- Claude はファイルまたはディレクトリを削除するコマンドを一切生成してはならない。
  例：rm, rm -rf, rm *, rmdir, unlink, cache --delete,
      lftp mirror --delete, rsync --delete, git clean -df, find -delete 等。

- 削除が必要な場合でも、Claude は削除コマンドを提案せず、
  「手動で削除してください」といった説明に留めること。

- 削除の推奨・削除操作の自動判断も禁止。

- ssh / lftp / デプロイ系スクリプトを生成する場合でも、
  削除コマンドの生成は禁止。

これらはすべての会話・コード生成に適用される。

---

# プロジェクト: minutter

**会議録音 → 議事録・ToDo・要約 自動作成システム（デモ版）**

## 技術スタック

| レイヤー | 技術 |
|---------|------|
| フレームワーク | Tauri v2（Rust + React/TypeScript） |
| バックエンドロジック | Rust |
| UI | React（TypeScript）+ Tailwind CSS |
| アイコン | Font Awesome のみ使用（絵文字禁止） |
| 音声キャプチャ | cpal（Rust クレート） |
| 音声変換 | ffmpeg-next（Rust バインディング）+ Tauri サイドカー |
| 音声認識 | Vosk-API Rust バインディング（日本語モデル同梱） |
| テキスト処理 | ルールベース（Rust で実装） |
| DB | SQLite（rusqlite） |
| 配布形式 | .msi（Windows）/ .dmg（macOS）/ .deb（Linux） |

## デモ版の制約

- **認証なし**（ローカルアプリのため不要）
- **外部 API 使用禁止**（ネットワーク越しの呼び出し・APIキーが必要なもの）
- **DB は SQLite 固定**（シングルユーザー）
- セッション管理・自動リセット・メンテナンスモード・Honeypot は実装しない
- Vosk モデル未配置時はエラー表示・ダウンロード URL 案内（アプリ内自動ダウンロード禁止）
- **フォールバック禁止**（モデル未配置 → 録音・文字起こし機能は使用不可）
- デプロイ先なし（インストーラー配布）

## ブランチ戦略

- **`main` ブランチでの直接作業禁止**
- `src/*` 以外の変更（設定ファイル等）は `main` ブランチへの直接 push を許可
- `src/*` の変更は必ず PR を作成すること
- PR には非エンジニア向けユーザーテスト手順を丁寧に記載すること

## 開発フロー（TDD 厳守）

```
plan → red test → coding → green test
```

- **テストフレームワーク**: Rust テスト（`#[cfg(test)]`）、Jest（React）、Playwright（E2E）
- フロントエンドの動作確認: `curl`、`wget --mirror`、Playwright
- **commit 前に必ず security review を実施すること**
- `.claude/OWASP10.md`、`.claude/QC10.md`、`.claude/TM.md`、`.claude/CC.md` を参照

## コーディング規約

- 制御構文・条件構文以外はクラスまたは関数に書くこと
- **グローバル変数禁止**（セキュリティの観点から）
- 文字列リテラルは設定ファイルに分離すること（ハードコード禁止）
- ハードコードをチェックするテストを書くこと
- **フォールバック禁止**。例外処理をしっかり書くこと
- デバッグトレースできるようにコードを書くこと
- `alert()` / `confirm()` / `prompt()` はプロジェクト全体で使用禁止
- SQLite 操作は rusqlite のプリペアドステートメント必須（SQL インジェクション対策）

## セキュリティ

- 個人情報は一切収集しない（ローカル保存のみ）
- OWASP10 チェック準拠
- DB ファイルは OS 標準の AppData ディレクトリに配置

## 言語

日本語のみ（デモ版）。

## エージェント構成

プロジェクトの規模に応じて以下のエージェントを使用する：

| エージェント | 役割 |
|---|---|
| `director` | 全体方針・意思決定 |
| `project-manager` | タスク・進捗管理 |
| `designer` | UI/UX・CRAP原則適用 |
| `debugger` | バグ調査・修正 |
| `tester` | テスト作成・実行 |
| `data-scientist` | データ分析・テキスト処理ロジック |
| `deployer` | インストーラービルド管理 |
| `writer` | ドキュメント・PR 文章 |
| `service-manager` | 品質・動作確認 |

### Sub Agent

- **pr-checker**: 全 PR を日本語化し、非エンジニア向けユーザーテストを PR 本文に丁寧に記載する
- **tester**: 全 PR 対象として、PR に書かれたユーザーテスト手順の実行スクリプトを `test/pr***/` に作成する（対象は開発サーバー）

---

## ディレクトリ構造

| ディレクトリ | 用途 |
|---|---|
| `TASKS/` | タスク管理 |
| `DEBUG/` | バグ報告 |
| `CLIENT/` | クライアント要望等 |
| `WORK/` | 作業報告 |
| `ENV/DEVELOPMENT.md` | 開発環境情報 |
| `ENV/PRODUCTION.md` | 本番環境情報 |
| `SPEC/` | 仕様書・設計図（ER図、DFD、シーケンス図、クラス図、状態遷移図、ユースケース図） |
| `DELETE/` | ゴミ箱（削除前の一時退避） |
| `test/pr***/` | PR ごとのテストスクリプト |

図解は Mermaid を使用すること。

---

## 参照チェックリスト

- `.claude/CC.md` — コンプライアンスチェック10項目
- `.claude/QC10.md` — 品質管理10項目
- `.claude/TM.md` — テストメソッドとフレームワーク
- `.claude/OWASP10.md` — OWASP Top 10（セキュリティ）
- `.claude/CRAP.md` — デザイン4か条
- `.claude/development-principles.md` — 開発原則（YAGNI・KISS・DRY・SOLID）
