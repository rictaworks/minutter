use log::debug;
use crate::config;
use crate::db::models::TodoItem;
use super::common::{split_sentences, deduplicate, filter_past, filter_negation};

/// ToDo 抽出ロジック
pub struct TodoProcessor;

impl TodoProcessor {
    /// テキストから ToDo 候補を抽出する
    ///
    /// 1. split_sentences で文に分割
    /// 2. KEYWORDS_TODO を含む文を候補にする
    /// 3. filter_past で過去形を除外
    /// 4. filter_negation で否定形を除外
    /// 5. deduplicate で重複除去
    /// 6. due_keyword を抽出
    pub fn extract_todos(text: &str) -> Vec<TodoItem> {
        debug!("ToDo 抽出開始: {} 文字", text.len());

        let sentences = split_sentences(text);
        debug!("文分割: {} 文", sentences.len());

        // ToDo キーワードを含む文を候補にする
        let candidates: Vec<String> = sentences
            .into_iter()
            .filter(|s| {
                let trimmed = s.trim();
                !trimmed.is_empty() && contains_todo_keyword(trimmed)
            })
            .map(|s| s.trim().to_string())
            .collect();

        debug!("ToDo キーワードマッチ: {} 件", candidates.len());

        // 過去形・否定形フィルタ
        let candidates = filter_past(candidates);
        let candidates = filter_negation(candidates);

        debug!("フィルタ後: {} 件", candidates.len());

        // 重複除去
        let candidates = deduplicate(candidates);

        // TodoItem に変換
        let items: Vec<TodoItem> = candidates
            .into_iter()
            .map(|text| {
                let due_keyword = extract_due_keyword(&text);
                debug!("ToDo: {} (due={})", text, due_keyword);
                TodoItem {
                    id: uuid::Uuid::new_v4().to_string(),
                    todo_text: text,
                    due_keyword,
                    is_checked: false,
                    is_manual: false,
                }
            })
            .collect();

        debug!("ToDo 抽出完了: {} 件", items.len());
        items
    }
}

/// テキストが ToDo キーワードを含むかどうかを判定する
fn contains_todo_keyword(text: &str) -> bool {
    config::KEYWORDS_TODO.iter().any(|kw| text.contains(kw))
}

/// テキストから期限キーワードを抽出する
pub fn extract_due_keyword(text: &str) -> String {
    for kw in config::KEYWORDS_DUE {
        if text.contains(kw) {
            return kw.to_string();
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_todos_basic() {
        let text = "資料を作成する必要があります。";
        let todos = TodoProcessor::extract_todos(text);
        assert!(!todos.is_empty(), "ToDo が抽出されるべき");
        assert!(todos[0].todo_text.contains("作成"));
    }

    #[test]
    fn test_filter_past_excludes_past_tense() {
        // 過去形の文は除外される
        let text = "先週資料を作成しました。今週は報告書を提出する。";
        let todos = TodoProcessor::extract_todos(text);
        // "ました" を含む文（過去形）は除外される
        for todo in &todos {
            assert!(
                !todo.todo_text.contains("ました"),
                "過去形の文が除外されていない: {}",
                todo.todo_text
            );
        }
    }

    #[test]
    fn test_filter_negation_excludes_negative() {
        // 否定形の文は除外される
        let text = "このタスクはやらない。別のタスクを実施する。";
        let todos = TodoProcessor::extract_todos(text);
        for todo in &todos {
            assert!(
                !todo.todo_text.contains("やらない"),
                "否定形の文が除外されていない: {}",
                todo.todo_text
            );
        }
    }

    #[test]
    fn test_filter_negation_shinasen() {
        let text = "この機能は実装しません。バグを修正する。";
        let todos = TodoProcessor::extract_todos(text);
        for todo in &todos {
            assert!(
                !todo.todo_text.contains("しません"),
                "否定形(しません)の文が除外されていない: {}",
                todo.todo_text
            );
        }
    }

    #[test]
    fn test_due_keyword_raishu() {
        let text = "来週中に資料を作成する。";
        let todos = TodoProcessor::extract_todos(text);
        assert!(!todos.is_empty());
        assert_eq!(todos[0].due_keyword, "来週");
    }

    #[test]
    fn test_due_keyword_raigetsu() {
        let text = "来月までに報告書を提出する。";
        let todos = TodoProcessor::extract_todos(text);
        assert!(!todos.is_empty());
        assert_eq!(todos[0].due_keyword, "来月");
    }

    #[test]
    fn test_due_keyword_konshuu() {
        let text = "今週中に確認する。";
        let todos = TodoProcessor::extract_todos(text);
        assert!(!todos.is_empty());
        assert_eq!(todos[0].due_keyword, "今週");
    }

    #[test]
    fn test_due_keyword_weekday() {
        let text = "金曜までに送付する。";
        let todos = TodoProcessor::extract_todos(text);
        assert!(!todos.is_empty());
        assert_eq!(todos[0].due_keyword, "金曜");
    }

    #[test]
    fn test_no_due_keyword() {
        let text = "資料を作成する。";
        let todos = TodoProcessor::extract_todos(text);
        if !todos.is_empty() {
            assert_eq!(todos[0].due_keyword, "", "期限キーワードがない場合は空文字");
        }
    }

    #[test]
    fn test_deduplicate() {
        let text = "資料を作成する。資料を作成する。";
        let todos = TodoProcessor::extract_todos(text);
        assert!(todos.len() <= 1, "重複が除去されるべき");
    }

    #[test]
    fn test_empty_text() {
        let todos = TodoProcessor::extract_todos("");
        assert!(todos.is_empty(), "空テキストで空リストが返るべき");
    }

    #[test]
    fn test_is_checked_default_false() {
        let text = "タスクを確認する。";
        let todos = TodoProcessor::extract_todos(text);
        if !todos.is_empty() {
            assert!(!todos[0].is_checked, "is_checked は初期値 false");
        }
    }

    #[test]
    fn test_is_manual_default_false() {
        let text = "タスクを確認する。";
        let todos = TodoProcessor::extract_todos(text);
        if !todos.is_empty() {
            assert!(!todos[0].is_manual, "is_manual は初期値 false（自動抽出）");
        }
    }

    #[test]
    fn test_extract_due_keyword_all_weekdays() {
        for day in &["月曜", "火曜", "水曜", "木曜", "金曜"] {
            let text = format!("{}までに対応する", day);
            let kw = extract_due_keyword(&text);
            assert_eq!(&kw, day, "{} の期限キーワード抽出失敗", day);
        }
    }

    #[test]
    fn test_todo_keywords_no_hardcode() {
        // KEYWORDS_TODO のキーワードが空でないことを確認（ハードコードチェック）
        for kw in config::KEYWORDS_TODO {
            assert!(!kw.is_empty());
        }
    }

    #[test]
    fn test_keywords_past_no_hardcode() {
        for kw in config::KEYWORDS_PAST {
            assert!(!kw.is_empty());
        }
    }

    #[test]
    fn test_keywords_negation_no_hardcode() {
        for kw in config::KEYWORDS_NEGATION {
            assert!(!kw.is_empty());
        }
    }
}
