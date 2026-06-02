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

## 技術スタック

- **フロントエンド**: Next.js（Vercel にデプロイ）
- **バックエンド**: Ruby on Rails（Render または Railway にデプロイ）
- **データベース**: PostgreSQL
- **補助 API**: FastAPI（AI・解析・画像加工）、Gin（高速並列・リアルタイム通信）
- **認証**: Google ログイン（OAuth2）
- **アイコン**: Font Awesome のみ使用（絵文字禁止）
- **環境変数**: 必ず `.env` を参照すること

## ブランチ戦略

- **`main` ブランチでの直接作業禁止**
- `src/*` 以外の変更（設定ファイル等）は `main` ブランチへの直接 push を許可
- `src/*` の変更は必ず PR を作成すること
- PR には非エンジニア向けユーザーテスト手順を丁寧に記載すること

## 開発フロー（TDD 厳守）

```
plan → red test → coding → green test
```

- **テストフレームワーク**: RSpec（Rails）、Jest（Next.js）、Playwright（E2E）
- フロントエンドの動作確認: `curl`、`wget --mirror`、Playwright
- **commit 前に必ず security review を実施すること**
- `.claude/OWASP10.md`、`.claude/QC10.md`、`.claude/TM.md`、`.claude/CC.md` を参照

## コーディング規約

- 制御構文・条件構文以外はクラスまたは関数に書くこと
- **グローバル変数禁止**（セキュリティの観点から）
- 文字列リテラルは設定ファイルまたはデータベースに分離すること
- ハードコードをチェックするテストを書くこと
- フォールバック禁止。例外処理をしっかり書くこと
- デバッグトレースできるようにコードを書くこと
- `alert()` / `confirm()` / `prompt()` はプロジェクト全体で使用禁止

## アーキテクチャ方針

- 規模に応じてマイクロサービス・MVC・API Gateway・メッセージングを意識する
- 安全なライブラリ・フレームワーク・OSS・SaaS を適用し車輪の再発明を避ける
- オリジナルコードを少なく保つこと

## 多言語対応

当初から以下7言語で開発すること：
- 日本語 / 英語 / フランス語 / 中国語 / ロシア語 / スペイン語 / アラビア語

**ただし開発者用管理画面は日本語のみ。**

## 環境判定

- 環境判定（development / staging / production）を必ず実装し分岐できるようにする
- 開発環境ではテスト可能にするため認証済み状態に分岐する

## デプロイ先

- **フロントエンド**: Vercel（無料プラン）
- **バックエンド・管理画面**: Render または Railway（無料プラン）
- **ドメイン**: rictaworks.jp のサブドメイン

## コンテンツ生成

- 画像は AI 生成を使用すること
- プロのライティングはライターエージェントに担当させること

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
| `test/pr***/` | PRごとのテストスクリプト |

図解は Mermaid を使用すること。

---

## エージェント構成

プロジェクトの規模に応じて以下のエージェントを使用する：

| エージェント | 役割 |
|---|---|
| `director` | 全体方針・意思決定 |
| `project-manager` | タスク・進捗管理 |
| `designer` | UI/UX・CRAP原則適用 |
| `debugger` | バグ調査・修正 |
| `tester` | テスト作成・実行 |
| `data-scientist` | データ分析・AI連携 |
| `deployer` | デプロイ・インフラ管理 |
| `writer` | プロのライティング・多言語対応 |
| `service-manager` | サービス運用・監視 |

### Sub Agent

- **pr-checker**: 全PRを日本語化し、非エンジニア向けユーザーテストをPR本文に丁寧に記載する
- **tester**: 全PR対象として、PRに書かれたユーザーテスト手順の実行スクリプトを `test/pr***/` に作成する（対象は開発サーバー）

---

## 参照チェックリスト

- `.claude/CC.md` — コンプライアンスチェック10項目
- `.claude/QC10.md` — 品質管理10項目
- `.claude/TM.md` — テストメソッドとフレームワーク
- `.claude/OWASP10.md` — OWASP Top 10（セキュリティ）
- `.claude/CRAP.md` — デザイン4か条
- `.claude/development-principles.md` — 開発原則（YAGNI・KISS・DRY・SOLID）
