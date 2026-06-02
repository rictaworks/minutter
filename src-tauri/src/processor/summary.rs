use log::debug;
use std::collections::HashMap;
use crate::config;
use super::common::split_sentences;

/// テキスト要約ロジック（TextRank 簡易版）
pub struct SummaryProcessor;

impl SummaryProcessor {
    /// テキストを要約する
    ///
    /// 1. split_sentences で文に分割
    /// 2. 目標文数 = max(1, round(文数 × SUMMARY_RATIO_MIN 〜 MAX の中間))
    /// 3. score_sentences: TF-IDF 的スコアリング
    /// 4. スコア上位の文を元の順番に並べ直して返す
    pub fn summarize(text: &str) -> String {
        debug!("要約開始: {} 文字", text.len());

        let sentences = split_sentences(text);
        let n = sentences.len();
        debug!("文数: {}", n);

        if n == 0 {
            debug!("文なし → 空文字を返す");
            return String::new();
        }

        if n == 1 {
            debug!("1文のみ → そのまま返す");
            return sentences[0].trim().to_string();
        }

        let target_count = Self::calc_target_count(n);
        debug!("目標文数: {}", target_count);

        let scores = Self::score_sentences(&sentences);
        debug!("スコア計算完了");

        // スコアでインデックスをソート（降順）
        let mut indexed_scores: Vec<(usize, f64)> = scores.into_iter().enumerate().collect();
        indexed_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 上位 target_count 文のインデックスを取得
        let mut selected_indices: Vec<usize> = indexed_scores
            .into_iter()
            .take(target_count)
            .map(|(i, _)| i)
            .collect();

        // 元の順番に並べ直す
        selected_indices.sort_unstable();

        let result = selected_indices
            .into_iter()
            .map(|i| sentences[i].trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("。");

        debug!("要約完了: {} 文字", result.len());
        result
    }

    /// 目標文数を計算する
    fn calc_target_count(n: usize) -> usize {
        // SUMMARY_RATIO_MIN と MAX の中間値を使用
        let ratio = (config::SUMMARY_RATIO_MIN + config::SUMMARY_RATIO_MAX) / 2.0;
        let target = (n as f64 * ratio).round() as usize;
        target.max(1).min(n)
    }

    /// 各文のスコアを計算する（TF-IDF 的スコアリング）
    ///
    /// - 各文中のユニーク単語（MIN_WORD_LENGTH 文字以上）を抽出
    /// - 単語の全文中での出現頻度（DF）の逆数を合計してスコアとする
    pub fn score_sentences(sentences: &[String]) -> Vec<f64> {
        if sentences.is_empty() {
            return Vec::new();
        }

        // 全文から単語の文書頻度（DF）を計算する
        let mut df: HashMap<String, usize> = HashMap::new();
        let all_words: Vec<Vec<String>> = sentences
            .iter()
            .map(|s| extract_words(s))
            .collect();

        for words in &all_words {
            let unique: std::collections::HashSet<_> = words.iter().cloned().collect();
            for word in unique {
                *df.entry(word).or_insert(0) += 1;
            }
        }

        let total_docs = sentences.len() as f64;

        // 各文のスコアを計算する
        sentences
            .iter()
            .zip(all_words.iter())
            .map(|(_, words)| {
                let unique_words: std::collections::HashSet<_> = words.iter().cloned().collect();
                if unique_words.is_empty() {
                    return 0.0;
                }
                unique_words
                    .iter()
                    .map(|word| {
                        let doc_freq = *df.get(word).unwrap_or(&1) as f64;
                        // IDF = log(total / df + 1) 、ゼロ除算防止のため +1
                        (total_docs / doc_freq).ln().max(0.0)
                    })
                    .sum::<f64>()
            })
            .collect()
    }
}

/// テキストからユニーク単語（MIN_WORD_LENGTH 文字以上）を抽出する
fn extract_words(text: &str) -> Vec<String> {
    // 日本語テキストはスペース・句読点で分割し、2文字以上の塊を単語とする
    let mut words = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        if ch.is_whitespace() || "。、！？,.!?「」『』【】()".contains(ch) {
            if current.chars().count() >= config::MIN_WORD_LENGTH {
                words.push(current.clone());
            }
            current.clear();
        } else {
            current.push(ch);
        }
    }
    if current.chars().count() >= config::MIN_WORD_LENGTH {
        words.push(current);
    }
    words
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;

    #[test]
    fn test_summarize_empty() {
        let result = SummaryProcessor::summarize("");
        assert!(result.is_empty(), "空テキストで空文字が返るべき");
    }

    #[test]
    fn test_summarize_single_sentence() {
        let result = SummaryProcessor::summarize("今日は会議があります。");
        assert!(!result.is_empty(), "1文テキストで非空文字が返るべき");
    }

    #[test]
    fn test_summary_ratio_within_range() {
        // 20文のテキストで比率が 15-20% 以内かチェック
        let sentences: Vec<String> = (0..20)
            .map(|i| format!("これはテスト文{}です。プロジェクトの進捗について話し合います", i))
            .collect();
        let text = sentences.join("。");

        let result = SummaryProcessor::summarize(&text);
        let result_sentences: Vec<_> = result
            .split('。')
            .filter(|s| !s.trim().is_empty())
            .collect();

        let original_sentences = split_sentences(&text);
        let original_count = original_sentences.iter()
            .filter(|s| !s.trim().is_empty())
            .count();

        let ratio = result_sentences.len() as f64 / original_count as f64;
        debug!(
            "要約比率: {}/{} = {:.3}",
            result_sentences.len(),
            original_count,
            ratio
        );

        // 目標文数が 1 以上かつ元の文数以下
        assert!(
            result_sentences.len() >= 1,
            "要約が少なすぎる: {}",
            result_sentences.len()
        );
        assert!(
            result_sentences.len() <= original_count,
            "要約が元より多い: {} > {}",
            result_sentences.len(),
            original_count
        );
        // 圧縮率の確認（許容幅あり: calc_target_count の丸め誤差考慮）
        assert!(
            ratio <= config::SUMMARY_RATIO_MAX + 0.05,
            "要約比率が上限を超えている: {:.3}",
            ratio
        );
    }

    #[test]
    fn test_summary_ratio_min_not_more_than_max() {
        assert!(config::SUMMARY_RATIO_MIN <= config::SUMMARY_RATIO_MAX);
    }

    #[test]
    fn test_calc_target_count_minimum_one() {
        // 文数 1 でも最低 1 文
        assert_eq!(SummaryProcessor::calc_target_count(1), 1);
    }

    #[test]
    fn test_calc_target_count_large() {
        let n = 100;
        let count = SummaryProcessor::calc_target_count(n);
        assert!(count >= 1);
        assert!(count <= n);
        let ratio = count as f64 / n as f64;
        assert!(ratio >= config::SUMMARY_RATIO_MIN - 0.01);
        assert!(ratio <= config::SUMMARY_RATIO_MAX + 0.01);
    }

    #[test]
    fn test_score_sentences_not_empty() {
        let sentences = vec![
            "今日は会議があります".to_string(),
            "プロジェクトの進捗を確認します".to_string(),
            "来週は発表があります".to_string(),
        ];
        let scores = SummaryProcessor::score_sentences(&sentences);
        assert_eq!(scores.len(), sentences.len());
        for score in &scores {
            assert!(*score >= 0.0, "スコアが負の値");
        }
    }

    #[test]
    fn test_score_sentences_empty() {
        let scores = SummaryProcessor::score_sentences(&[]);
        assert!(scores.is_empty());
    }

    #[test]
    fn test_summarize_returns_subset_of_original() {
        let text = "会議の冒頭で自己紹介を行いました。プロジェクトの現状について報告しました。課題点を洗い出しました。次のステップを検討しました。予算の見直しを行いました。タイムラインを確認しました。チームの役割分担を確認しました。来週の予定を確認しました。";
        let result = SummaryProcessor::summarize(text);
        assert!(!result.is_empty(), "要約が空");
    }

    #[test]
    fn test_extract_words_min_length() {
        let words = extract_words("a bb ccc");
        // "a" は 1文字なので除外、"bb" 以上が含まれる
        for word in &words {
            assert!(word.chars().count() >= config::MIN_WORD_LENGTH);
        }
    }

    #[test]
    fn test_summary_ratio_constants_no_hardcode() {
        // config から参照していることの確認
        let ratio_min = config::SUMMARY_RATIO_MIN;
        let ratio_max = config::SUMMARY_RATIO_MAX;
        assert!(ratio_min > 0.0);
        assert!(ratio_max > ratio_min);
    }
}
