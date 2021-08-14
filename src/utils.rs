use crate::implicit_search_request_parser::ImplicitPaperSearchRequest;
use crate::storage::Paper;
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
        .expect("Cannot build a regular expression");
    }

    RE.replace_all(text, r#"\$symbol"#).to_string()
}

pub fn markdown_v2_escape_inline_uri(text: &str) -> String {
    lazy_static! {
        static ref SYMBOLS_FOR_ESCAPING: [&'static str; 2] = [")", "\\"];
        static ref RE: Regex = Regex::new(
            format!(r#"(?P<symbol>([\{}]))"#, &SYMBOLS_FOR_ESCAPING.join(r#"\"#)).as_str()
        )
        .expect("Cannot build a regular expression");
    }

    RE.replace_all(text, r#"\$symbol"#).to_string()
}

pub fn convert_papers_to_result(papers: Vec<Paper>) -> String {
    let mut formatted_papers = Vec::<String>::new();
    formatted_papers.reserve(papers.len());

    for paper in papers.iter() {
        formatted_papers.push(paper.format_with_markdownv2())
    }

    formatted_papers.sort_unstable();

    return formatted_papers.join("\n\n");
}

pub fn find_search_request_in_message(
    text: &str,
) -> anyhow::Result<Vec<ImplicitPaperSearchRequest>> {
    let result = crate::implicit_search_request_parser::many_paper_requests(text);

    match result {
        Ok((_, papers)) => Ok(papers),
        Err(err) => Err(anyhow::anyhow!("Cannot parse search request: {}", err)),
    }
}
