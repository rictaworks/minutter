use crate::config;

/// テキストを文に分割する
/// 句読点・改行・感嘆符・疑問符で分割する
pub fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if config::SENTENCE_DELIMITERS.contains(&ch) {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
            current.clear();
        }
    }

    // 末尾の残りを追加する
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        sentences.push(trimmed);
    }

    sentences
}

/// 過去形を含む文を除外する
pub fn filter_past(sentences: Vec<String>) -> Vec<String> {
    sentences
        .into_iter()
        .filter(|s| {
            !config::KEYWORDS_PAST.iter().any(|kw| s.contains(kw))
        })
        .collect()
}

/// 否定形を含む文を除外する
pub fn filter_negation(sentences: Vec<String>) -> Vec<String> {
    sentences
        .into_iter()
        .filter(|s| {
            !config::KEYWORDS_NEGATION.iter().any(|kw| s.contains(kw))
        })
        .collect()
}

/// 重複する文を除去する（順序保持）
pub fn deduplicate(items: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_sentences_kuten() {
        let text = "これはテストです。次の文です。";
        let sents = split_sentences(text);
        assert_eq!(sents.len(), 2);
    }

    #[test]
    fn test_split_sentences_newline() {
        let text = "一行目\n二行目";
        let sents = split_sentences(text);
        assert_eq!(sents.len(), 2);
    }

    #[test]
    fn test_split_sentences_empty() {
        let sents = split_sentences("");
        assert!(sents.is_empty());
    }

    #[test]
    fn test_split_sentences_no_delimiter() {
        let text = "区切りなしのテキスト";
        let sents = split_sentences(text);
        assert_eq!(sents.len(), 1);
        assert_eq!(sents[0], text);
    }

    #[test]
    fn test_filter_past_removes_past() {
        let sentences = vec![
            "昨日資料を作成した。".to_string(),
            "今日は報告する。".to_string(),
            "先週会議でした。".to_string(),
        ];
        let result = filter_past(sentences);
        assert_eq!(result.len(), 1);
        assert!(result[0].contains("報告する"));
    }

    #[test]
    fn test_filter_past_keeps_non_past() {
        let sentences = vec![
            "今日は作業する。".to_string(),
            "明日は確認する。".to_string(),
        ];
        let result = filter_past(sentences);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filter_negation_removes_negative() {
        let sentences = vec![
            "タスクをやらない。".to_string(),
            "対応しない。".to_string(),
            "報告する。".to_string(),
        ];
        let result = filter_negation(sentences);
        assert_eq!(result.len(), 1);
        assert!(result[0].contains("報告する"));
    }

    #[test]
    fn test_filter_negation_keeps_positive() {
        let sentences = vec!["タスクをやる。".to_string()];
        let result = filter_negation(sentences);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_deduplicate_removes_duplicates() {
        let items = vec![
            "タスクA".to_string(),
            "タスクB".to_string(),
            "タスクA".to_string(),
        ];
        let result = deduplicate(items);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "タスクA");
        assert_eq!(result[1], "タスクB");
    }

    #[test]
    fn test_deduplicate_preserves_order() {
        let items = vec![
            "C".to_string(),
            "A".to_string(),
            "B".to_string(),
            "A".to_string(),
        ];
        let result = deduplicate(items);
        assert_eq!(result, vec!["C", "A", "B"]);
    }

    #[test]
    fn test_split_sentences_mixed_delimiters() {
        let text = "質問ですか？はい！確認します。";
        let sents = split_sentences(text);
        assert_eq!(sents.len(), 3);
    }
}
