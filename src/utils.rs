use lazy_static::lazy_static;
use regex::Regex;

pub fn markdown_v2_escape(text: &str) -> String {
    lazy_static! {
        static ref TELEGRAM_ESCAPE_REGEX_ESCAPE: [&'static str; 11] =
            ["{", "}", "[", "]", "(", ")", "+", "*", "|", ".", "-"];
        static ref TELEGRAM_ESCAPE_REGEX_NOT_ESCAPE: [&'static str; 7] =
            ["_", "~", "`", ">", "#", "=", "!"];
        static ref RE: Regex = Regex::new(
            format!(
                r#"(?P<symbol>([\{}{}]))"#,
                &TELEGRAM_ESCAPE_REGEX_ESCAPE.join(r#"\"#),
                &TELEGRAM_ESCAPE_REGEX_NOT_ESCAPE.join("")
            )
            .as_str()
        )
        .unwrap();
    }

    RE.replace_all(text, r#"\$symbol"#).to_string()
}

pub fn markdown_v2_escape_inline_uri(text: &str) -> String {
    lazy_static! {
        static ref SYMBOLS_FOR_ESCAPING: [&'static str; 2] = [")", "\\"];
        static ref RE: Regex = Regex::new(
            format!(r#"(?P<symbol>([\{}]))"#, &SYMBOLS_FOR_ESCAPING.join(r#"\"#)).as_str()
        )
        .unwrap();
    }

    RE.replace_all(text, r#"\$symbol"#).to_string()
}
