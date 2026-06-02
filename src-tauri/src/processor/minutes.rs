use log::debug;
use crate::config;
use crate::db::models::MinuteItem;
use super::common::{split_sentences, deduplicate, filter_past, filter_negation};

/// 議事録生成ロジック
pub struct MinutesProcessor;

impl MinutesProcessor {
    /// テキストから議事録アイテムを生成する
    ///
    /// 1. `split_sentences` でテキストを文に分割
    /// 2. KEYWORDS_DECISION → decisions セクション
    ///    KEYWORDS_NEXT     → next セクション
    ///    その他             → body セクション
    /// 3. body のみ filter_past / filter_negation を適用
    /// 4. deduplicate で重複除去
    pub fn generate_minutes(text: &str) -> Vec<MinuteItem> {
        debug!("議事録生成開始: {} 文字", text.len());

        let sentences = split_sentences(text);
        debug!("文分割: {} 文", sentences.len());

        let mut decisions: Vec<String> = Vec::new();
        let mut next: Vec<String> = Vec::new();
        let mut body: Vec<String> = Vec::new();

        for sentence in sentences {
            let trimmed = sentence.trim().to_string();
            if trimmed.is_empty() {
                continue;
            }

            if contains_any_keyword(&trimmed, config::KEYWORDS_DECISION) {
                debug!("decisions: {}", trimmed);
                decisions.push(trimmed);
            } else if contains_any_keyword(&trimmed, config::KEYWORDS_NEXT) {
                debug!("next: {}", trimmed);
                next.push(trimmed);
            } else {
                body.push(trimmed);
            }
        }

        // body のみフィルタ適用
        let body = filter_past(body);
        let body = filter_negation(body);

        let decisions = deduplicate(decisions);
        let next = deduplicate(next);
        let body = deduplicate(body);

        let mut items: Vec<MinuteItem> = Vec::new();
        let mut sort_order: i64 = 0;

        for content in decisions {
            items.push(MinuteItem {
                id: uuid::Uuid::new_v4().to_string(),
                section_type: config::SECTION_DECISIONS.to_string(),
                content,
                sort_order,
            });
            sort_order += 1;
        }

        for content in next {
            items.push(MinuteItem {
                id: uuid::Uuid::new_v4().to_string(),
                section_type: config::SECTION_NEXT.to_string(),
                content,
                sort_order,
            });
            sort_order += 1;
        }

        for content in body {
            items.push(MinuteItem {
                id: uuid::Uuid::new_v4().to_string(),
                section_type: config::SECTION_BODY.to_string(),
                content,
                sort_order,
            });
            sort_order += 1;
        }

        debug!("議事録生成完了: {} 件", items.len());
        items
    }
}

/// いずれかのキーワードを含むかどうかを判定する
pub fn contains_any_keyword(text: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|kw| text.contains(kw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decisions_keyword_match() {
        let text = "予算について決定しました。次回の議題を確認します。";
        let items = MinutesProcessor::generate_minutes(text);
        let decisions: Vec<_> = items
            .iter()
            .filter(|i| i.section_type == config::SECTION_DECISIONS)
            .collect();
        assert!(!decisions.is_empty(), "decisions が生成されるべき");
        assert!(decisions[0].content.contains("決定"));
    }

    #[test]
    fn test_next_keyword_match() {
        let text = "次回の議題は製品ロードマップです。";
        let items = MinutesProcessor::generate_minutes(text);
        let next: Vec<_> = items
            .iter()
            .filter(|i| i.section_type == config::SECTION_NEXT)
            .collect();
        assert!(!next.is_empty(), "next が生成されるべき");
        assert!(next[0].content.contains("次回"));
    }

    #[test]
    fn test_body_fallthrough() {
        let text = "今日は天気が良いです。";
        let items = MinutesProcessor::generate_minutes(text);
        let body: Vec<_> = items
            .iter()
            .filter(|i| i.section_type == config::SECTION_BODY)
            .collect();
        assert!(!body.is_empty(), "body が生成されるべき");
    }

    #[test]
    fn test_body_past_filter() {
        // 過去形はbodyから除外される
        let text = "昨日会議をしていた。今日は新しいタスクがある。";
        let items = MinutesProcessor::generate_minutes(text);
        let body: Vec<_> = items
            .iter()
            .filter(|i| i.section_type == config::SECTION_BODY)
            .collect();
        // "していた" を含む文は除外される
        for item in &body {
            assert!(
                !item.content.contains("していた"),
                "過去形の文が body に残っている: {}",
                item.content
            );
        }
    }

    #[test]
    fn test_body_negation_filter() {
        // 否定形はbodyから除外される
        let text = "このタスクはしない。別のタスクをやる。";
        let items = MinutesProcessor::generate_minutes(text);
        let body: Vec<_> = items
            .iter()
            .filter(|i| i.section_type == config::SECTION_BODY)
            .collect();
        for item in &body {
            assert!(
                !item.content.contains("しない"),
                "否定形の文が body に残っている: {}",
                item.content
            );
        }
    }

    #[test]
    fn test_sort_order_sequential() {
        let text = "予算を決定しました。次回の議題はロードマップです。実装を進めます。";
        let items = MinutesProcessor::generate_minutes(text);
        for (i, item) in items.iter().enumerate() {
            assert_eq!(item.sort_order, i as i64, "sort_order が連番でない");
        }
    }

    #[test]
    fn test_deduplicate_in_minutes() {
        let text = "予算を決定しました。予算を決定しました。";
        let items = MinutesProcessor::generate_minutes(text);
        let decisions: Vec<_> = items
            .iter()
            .filter(|i| i.section_type == config::SECTION_DECISIONS)
            .collect();
        assert_eq!(decisions.len(), 1, "重複が除去されるべき");
    }

    #[test]
    fn test_empty_text() {
        let items = MinutesProcessor::generate_minutes("");
        assert!(items.is_empty(), "空テキストで空リストが返るべき");
    }

    #[test]
    fn test_contains_any_keyword_true() {
        assert!(contains_any_keyword("これを決定します", config::KEYWORDS_DECISION));
    }

    #[test]
    fn test_contains_any_keyword_false() {
        assert!(!contains_any_keyword("今日は晴れです", config::KEYWORDS_DECISION));
    }

    #[test]
    fn test_section_type_values_match_config() {
        // セクションタイプがconfig定数と一致するかチェック（ハードコード禁止）
        let text = "予算を決定。次回の議題。タスク実施。";
        let items = MinutesProcessor::generate_minutes(text);
        for item in &items {
            let valid = item.section_type == config::SECTION_DECISIONS
                || item.section_type == config::SECTION_NEXT
                || item.section_type == config::SECTION_BODY;
            assert!(valid, "不明なセクションタイプ: {}", item.section_type);
        }
    }
}
