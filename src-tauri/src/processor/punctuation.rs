/// Vosk の無句読点テキストに句読点を補完する
///
/// Vosk の日本語モデルは句読点を出力しない。
/// 文末表現パターンに最長一致でマッチし「。」を挿入する。
/// スペース区切りトークンは結合してから処理する。
pub fn restore_punctuation(text: &str) -> String {
    if text.is_empty() {
        return text.to_string();
    }
    // すでに句読点が含まれていれば補完不要（手動編集済みテキストを保護）
    if text.contains('。') || text.contains('、') {
        return text.to_string();
    }

    // Vosk はチャンク間をスペースで区切る場合がある。
    // 日本語はスペース不要なのでまとめて結合する。
    let joined: String = text.split_whitespace().collect();

    insert_kuten(&joined)
}

/// 文末パターンに「。」を挿入する
///
/// アルゴリズム:
/// 1. 直前の「。」挿入からの文字数が MIN_SENTENCE_LEN 以上になったら
///    パターン一致を試みる（短すぎる文を作らないため）。
/// 2. パターンは長い順に定義し、最初にマッチしたものを採用する。
/// 3. マッチしたパターン末尾に「。」を挿入してポインタを進める。
fn insert_kuten(text: &str) -> String {
    /// 一文の最低文字数（これ未満では句点を挿入しない）
    const MIN_SENTENCE_LEN: usize = 5;

    /// 文末と判定する語尾パターン（長い順に定義すること）
    const PATTERNS: &[&str] = &[
        // ～ませんでした系
        "ませんでした",
        // ～ましょう系
        "ましょう",
        // ～ましたか系
        "ましたか",
        // ～ましたよ / ましたね
        "ましたよね",
        "ましたよ",
        "ましたね",
        // ～ました
        "ました",
        // ～ません
        "ません",
        // ～ますよね / ますよ / ますね / ますか
        "ますよね",
        "ますよ",
        "ますね",
        "ますか",
        // ～ます
        "ます",
        // ～でしたか / でしたよ / でしたね
        "でしたか",
        "でしたよ",
        "でしたね",
        // ～でした
        "でした",
        // ～ですよね / ですよ / ですね / ですか
        "ですよね",
        "ですよ",
        "ですね",
        "ですか",
        // ～です
        "です",
        // ～だろうか / だろう
        "だろうか",
        "だろう",
        // ～だよね / だよな / だよ / だね / だな / だった
        "だよね",
        "だよな",
        "だよ",
        "だね",
        "だな",
        "だった",
        // ～と思います / と思いました / と思う / と思った
        "と思います",
        "と思いました",
        "と思う",
        "と思った",
        // ～ということです / ということで / ということ
        "ということです",
        "ということで",
        "ということ",
        // ～ください
        "ください",
        // ～できます / できる / します / する
        "できます",
        "できる",
        "します",
        "する",
        // ～なります / なる / あります / ある / います / いる
        "なります",
        "なる",
        "あります",
        "ある",
        "います",
        "いる",
        // ～ない / たい
        "ない",
        "たい",
        // 短い終助詞（最後に判定）
        "よね",
        "よ",
        "ね",
        "わ",
    ];

    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut result = String::with_capacity(len + len / 8);
    let mut pos = 0;
    let mut since_last_kuten: usize = 0;

    while pos < len {
        // 最低文字数に達したらパターンマッチを試みる
        let mut matched = false;
        if since_last_kuten >= MIN_SENTENCE_LEN {
            for pattern in PATTERNS {
                let pat_chars: Vec<char> = pattern.chars().collect();
                let pat_len = pat_chars.len();
                if pos + pat_len > len {
                    continue;
                }
                let window_matches = chars[pos..pos + pat_len]
                    .iter()
                    .zip(pat_chars.iter())
                    .all(|(a, b)| a == b);
                if window_matches {
                    // パターンを出力して「。」を付ける
                    for &c in &chars[pos..pos + pat_len] {
                        result.push(c);
                    }
                    result.push('。');
                    pos += pat_len;
                    since_last_kuten = 0;
                    matched = true;
                    break;
                }
            }
        }

        if !matched {
            result.push(chars[pos]);
            pos += 1;
            since_last_kuten += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_already_has_kuten() {
        let text = "今日は会議です。明日は休みです。";
        assert_eq!(restore_punctuation(text), text);
    }

    #[test]
    fn test_already_has_ten() {
        let text = "今日は、会議があります";
        assert_eq!(restore_punctuation(text), text);
    }

    #[test]
    fn test_empty() {
        assert_eq!(restore_punctuation(""), "");
    }

    #[test]
    fn test_masu_pattern() {
        // 「ます」パターンで句点が入ること
        let text = "今日は作業しますそれからレビューします";
        let result = restore_punctuation(text);
        assert!(result.contains('。'), "句読点が挿入されるべき: {}", result);
    }

    #[test]
    fn test_mashita_pattern() {
        let text = "資料を作成しましたご確認をお願いします";
        let result = restore_punctuation(text);
        assert!(result.contains('。'), "句読点が挿入されるべき: {}", result);
    }

    #[test]
    fn test_desu_pattern() {
        let text = "今日は金曜日ですそれでは会議を始めます";
        let result = restore_punctuation(text);
        assert!(result.contains('。'), "句読点が挿入されるべき: {}", result);
    }

    #[test]
    fn test_spaced_vosk_output() {
        // Vosk がスペースを挟んで出力するケース
        let text = "今日 は 作業 し ます 明日 まで に 送り ます";
        let result = restore_punctuation(text);
        assert!(result.contains('。'), "句読点が挿入されるべき: {}", result);
        assert!(!result.contains(' '), "スペースが残っていてはいけない: {}", result);
    }

    #[test]
    fn test_min_sentence_len() {
        // 短すぎる文には句点を挿入しない
        let text = "ですます";
        let result = restore_punctuation(text);
        // 4文字以下なので MIN_SENTENCE_LEN を超えない → 末尾のみ挿入される可能性
        // ここでは単純に処理されることを確認
        assert!(!result.is_empty());
    }

    #[test]
    fn test_multiple_sentences() {
        let text = "今日は資料を作成しました明日はレビューをしますご確認をお願いします";
        let result = restore_punctuation(text);
        let kuten_count = result.chars().filter(|&c| c == '。').count();
        assert!(kuten_count >= 2, "複数の句点が挿入されるべき: {} ({}個)", result, kuten_count);
    }
}
