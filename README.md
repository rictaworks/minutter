# minutter

## 自動ログイン

開発環境では Google ログインをスキップし、自動的に認証済み状態で動作します。

| 環境変数 | 値 | 動作 |
|---|---|---|
| `NEXT_PUBLIC_APP_ENV` | `development` | 認証スキップ（開発者用テストユーザーで自動ログイン） |
| `NEXT_PUBLIC_APP_ENV` | `production` | Google OAuth2 ログインが必要 |

---

## ページ一覧

> 設計書受領後に更新します。

| ページ名 | URL |
|---|---|
| （設計書待ち） | - |

---

## API 一覧

> 設計書受領後に更新します。仕様書は `SPEC/API仕様書.md` を参照。

| タイトル | エンドポイント |
|---|---|
| （設計書待ち） | - |

---

## 技術スタック

| レイヤー | 技術 |
|---|---|
| フロントエンド | Next.js → Vercel |
| バックエンド | Ruby on Rails → Render / Railway |
| データベース | PostgreSQL |
| 補助 API（AI/解析） | FastAPI |
| 補助 API（リアルタイム） | Gin |
| 認証 | Google OAuth2 |
| アイコン | Font Awesome |

## ドメイン

- フロントエンド: `minutter.rictaworks.jp`
- バックエンド API: `api.minutter.rictaworks.jp`
- 管理画面: `admin.minutter.rictaworks.jp`

## 開発者向けドキュメント

- [開発環境](ENV/DEVELOPMENT.md)
- [本番環境](ENV/PRODUCTION.md)
- [仕様書](SPEC/)
- [タスク](TASKS/)
- [バグ報告](DEBUG/)
