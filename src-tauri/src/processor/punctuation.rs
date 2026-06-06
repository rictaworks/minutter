/// Vosk の無句読点テキストに句読点を補完する
///
/// Vosk の日本語モデルは句読点を出力しないため、
/// 文末表現パターンに基づいてルールベースで「。」を付与する。
pub fn restore_punctuation(text: &str) -> String {
    if text.is_empty() {
        return text.to_string();
    }

    // すでに句読点が含まれていれば補完不要
    if text.contains('。') || text.contains('、') {
        return text.to_string();
    }

    // Vosk は単語をスペース区切りで返すことがあるため、スペースを除去してから処理する
    // ただし英数字間のスペースは保持する（半角スペースを全角スペースに変換後、日本語部分のみ結合）
    let text = text.replace('\u{3000}', " "); // 全角スペース → 半角スペース

    // スペース区切りトークンを結合する（日本語はスペース不要）
    let joined = join_japanese_tokens(&text);

    // 文末表現で区切り「。」を付与する
    insert_kuten(&joined)
}

/// スペース区切りの日本語トークンを結合する
/// 英数字・記号を含む部分はスペースを保持する
fn join_japanese_tokens(text: &str) -> String {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    if tokens.len() <= 1 {
        return text.to_string();
    }

    let mut result = String::new();
    for (i, token) in tokens.iter().enumerate() {
        if i == 0 {
            result.push_str(token);
            continue;
        }
        let prev_last = result.chars().last().unwrap_or(' ');
        let curr_first = token.chars().next().unwrap_or(' ');

        // 前後どちらかが ASCII なら空白を保持
        if prev_last.is_ascii() || curr_first.is_ascii() {
            result.push(' ');
        }
        result.push_str(token);
    }
    result
}

/// 文末表現パターンに基づいて「。」を挿入する
fn insert_kuten(text: &str) -> String {
    // 文末と判定するサフィックスパターン（長い順に並べる）
    const SENTENCE_END_PATTERNS: &[&str] = &[
        // ～ました系
        "ました",
        "でした",
        "ませんでした",
        "ましたか",
        // ～ます系
        "ます",
        "ません",
        "ますか",
        "ましょう",
        // ～です系
        "です",
        "ですか",
        "ですね",
        "ですよ",
        "ですよね",
        // ～だ系
        "だ",
        "だな",
        "だね",
        "だよ",
        "だよな",
        "だよね",
        "だろう",
        "だろうか",
        "だった",
        // ～る系（動詞終止形）
        "する",
        "できる",
        "なる",
        "ある",
        "いる",
        "くる",
        "おく",
        "もらう",
        "あげる",
        "もらえる",
        // ～て系（接続表現・一時停止）
        "て",
        "で",
        "ね",
        "よ",
        "な",
        "わ",
        "か",
        // ～い系（形容詞終止形）
        "ない",
        "たい",
        "よい",
        "いい",
        "多い",
        "少ない",
        // ～と思います系
        "と思います",
        "と思いました",
        "と思う",
        "と思った",
        "と思って",
        // ～ください系
        "ください",
        "くださいね",
        // ～という系
        "という",
        "ということ",
        "ということで",
        "ということです",
        // ～について系
        "について",
        "に関して",
        "に関しては",
        // ～ので系
        "ので",
        "から",
        "けど",
        "けれど",
        "が",
    ];

    // チャンク（区切り候補の単位）に分割する
    // アプローチ: 文末パターンの直後に「。」を差し込む
    // まず全体を走査しながら文末候補位置を見つける

    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    if len == 0 {
        return text.to_string();
    }

    let mut result = String::new();
    let mut pos = 0;

    while pos < len {
        // 現在位置以降の残りテキストを文字列に
        let remaining: String = chars[pos..].iter().collect();

        // 最長一致で文末パターンを探す
        // パターンを長さ降順に探す（SENTENCE_END_PATTERNS はすでに長い順）
        let mut matched_end: Option<usize> = None;
        for pattern in SENTENCE_END_PATTERNS {
            let pat_chars: Vec<char> = pattern.chars().collect();
            let pat_len = pat_chars.len();
            if pos + pat_len > len {
                continue;
            }
            let window: String = chars[pos..pos + pat_len].iter().collect();
            if window == *pattern {
                // パターン末尾位置
                let end_pos = pos + pat_len;
                // 直後が文末記号・空白・文字列末尾 or 別の文末パターン開始なら採用
                let next_is_boundary = end_pos >= len
                    || chars[end_pos] == ' '
                    || chars[end_pos] == '\n'
                    || chars[end_pos] == '　'
                    || "、。！？,.".contains(chars[end_pos]);
                if next_is_boundary {
                    matched_end = Some(end_pos);
                    break;
                }
            }
        }

        if let Some(end) = matched_end {
            // パターン部分をそのまま出力して「。」を付ける
            let segment: String = chars[pos..end].iter().collect();
            result.push_str(&segment);
            // 次の文字が既に句読点・改行でなければ「。」を付ける
            let next_char = if end < len { Some(chars[end]) } else { None };
            if next_char != Some('。')
                && next_char != Some('！')
                && next_char != Some('？')
                && next_char != Some('\n')
            {
                result.push('。');
            }
            pos = end;
        } else {
            // マッチなし: 1文字そのまま出力
            result.push(chars[pos]);
            pos += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_already_has_punctuation() {
        let text = "今日は会議です。明日は休みです。";
        assert_eq!(restore_punctuation(text), text);
    }

    #[test]
    fn test_empty() {
        assert_eq!(restore_punctuation(""), "");
    }

    #[test]
    fn test_join_tokens() {
        let text = "今日 は 会議 です 明日 は 休み です";
        let result = restore_punctuation(text);
        assert!(result.contains('。'), "句読点が挿入されるべき: {}", result);
    }

    #[test]
    fn test_insert_kuten_masu() {
        let text = "今日は作業しますそれからレビューします";
        let result = restore_punctuation(text);
        assert!(result.contains('。'), "句読点が挿入されるべき: {}", result);
    }

    #[test]
    fn test_vosk_spaced_output() {
        let text = "今日 は 資料 を 作成 し ます 明日 まで に 送り ます";
        let result = restore_punctuation(text);
        assert!(result.contains('。'), "句読点が挿入されるべき: {}", result);
    }
}
