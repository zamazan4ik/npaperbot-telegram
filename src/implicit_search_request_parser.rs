use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_while, take_while1};
use nom::character::is_digit;
use nom::combinator::opt;
use nom::error::VerboseError;
use nom::multi::many0;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug, Eq, PartialEq)]
pub struct ImplicitPaperSearchRequest {
    pub paper_type: String,
    pub paper_number: String,
    pub revision_number: Option<i32>,
}

pub fn is_char_digit(chr: char) -> bool {
    return chr.is_ascii() && is_digit(chr as u8);
}

fn is_not_start_request_character(chr: char) -> bool {
    !(chr == '[' || chr == '{' || chr == '<')
}

fn start_paper_search_request(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    alt::<_, _, nom::error::VerboseError<&str>, _>((tag("["), tag("{"), tag("<")))(input)
}

fn end_paper_search_request(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    alt::<_, _, nom::error::VerboseError<&str>, _>((tag("]"), tag("}"), tag(">")))(input)
}

fn paper_type(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    alt::<_, _, nom::error::VerboseError<&str>, _>((
        tag_no_case("N"),
        tag_no_case("P"),
        tag_no_case("D"),
        tag_no_case("CWG"),
        tag_no_case("EWG"),
        tag_no_case("LWG"),
        tag_no_case("LEWG"),
        tag_no_case("FS"),
        tag_no_case("EDIT"),
        tag_no_case("SD"),
    ))(input)
}

fn paper_number(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    take_while1(is_char_digit)(input)
}

fn revision(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    tag_no_case("R")(input)
}

fn revision_number(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    take_while1(is_char_digit)(input)
}

fn paper(input: &str) -> IResult<&str, ImplicitPaperSearchRequest, VerboseError<&str>> {
    let (input, (paper_type, paper_number, _, revision_number)) = tuple((
        paper_type,
        paper_number,
        opt(revision),
        opt(revision_number),
    ))(input)?;

    if let Some(revision_number) = revision_number {
        Ok((
            input,
            ImplicitPaperSearchRequest {
                paper_type: paper_type.to_string(),
                paper_number: paper_number.to_string(),
                revision_number: Some(revision_number.parse().expect("Cannot parse as i32")),
            },
        ))
    } else {
        Ok((
            input,
            ImplicitPaperSearchRequest {
                paper_type: paper_type.to_string(),
                paper_number: paper_number.to_string(),
                revision_number: None,
            },
        ))
    }
}

pub fn paper_request(input: &str) -> IResult<&str, ImplicitPaperSearchRequest, VerboseError<&str>> {
    let (input, (_, parsed_paper, _)) =
        tuple((start_paper_search_request, paper, end_paper_search_request))(input)?;
    Ok((input, parsed_paper))
}

pub fn paper_request_with_leading_trash(
    input: &str,
) -> IResult<&str, ImplicitPaperSearchRequest, VerboseError<&str>> {
    let (input, (_, paper_request)) =
        tuple((take_while(is_not_start_request_character), paper_request))(input)?;

    Ok((input, paper_request))
}

pub fn many_paper_requests(
    input: &str,
) -> IResult<&str, Vec<ImplicitPaperSearchRequest>, VerboseError<&str>> {
    many0(paper_request_with_leading_trash)(input)
}

#[cfg(test)]
mod tests {
    use crate::implicit_search_request_parser::{
        end_paper_search_request, is_char_digit, is_not_start_request_character,
        many_paper_requests, paper, paper_number, paper_request, paper_request_with_leading_trash,
        paper_type, revision, revision_number, start_paper_search_request,
        ImplicitPaperSearchRequest,
    };
    use nom::error::ErrorKind::{Alt, Tag, TakeWhile1};
    use nom::error::VerboseError;
    use nom::error::VerboseErrorKind::Nom;
    use nom::lib::std::result::Result::Err;

    #[test]
    fn test_is_char_digit() {
        assert!(is_char_digit('1'));
        assert!(!is_char_digit('a'));
        assert!(!is_char_digit('Д'));
    }

    #[test]
    fn test_is_not_start_request_character() {
        assert!(is_not_start_request_character('a'));
        assert!(!is_not_start_request_character('['));
        assert!(!is_not_start_request_character('{'));
        assert!(!is_not_start_request_character('<'));
    }

    #[test]
    fn test_start_paper_search_request() {
        assert_eq!(start_paper_search_request("["), Ok(("", "[")));
        assert_eq!(start_paper_search_request("{"), Ok(("", "{")));
        assert_eq!(start_paper_search_request("<"), Ok(("", "<")));

        assert_eq!(
            start_paper_search_request("("),
            Err(nom::Err::Error(VerboseError {
                errors: vec![("(", Nom(Tag)), ("(", Nom(Alt))]
            }))
        );
    }

    #[test]
    fn test_end_paper_search_request() {
        assert_eq!(end_paper_search_request("]"), Ok(("", "]")));
        assert_eq!(end_paper_search_request("}"), Ok(("", "}")));
        assert_eq!(end_paper_search_request(">"), Ok(("", ">")));

        assert_eq!(
            end_paper_search_request(")"),
            Err(nom::Err::Error(VerboseError {
                errors: vec![(")", Nom(Tag)), (")", Nom(Alt))]
            }))
        );
    }

    #[test]
    fn test_paper_type() {
        assert_eq!(paper_type("N"), Ok(("", "N")));
        assert_eq!(paper_type("n"), Ok(("", "n")));
        assert_eq!(paper_type("P"), Ok(("", "P")));
        assert_eq!(paper_type("D"), Ok(("", "D")));
        assert_eq!(paper_type("CWG"), Ok(("", "CWG")));
        assert_eq!(paper_type("EWG"), Ok(("", "EWG")));
        assert_eq!(paper_type("LWG"), Ok(("", "LWG")));
        assert_eq!(paper_type("LEWG"), Ok(("", "LEWG")));
        assert_eq!(paper_type("FS"), Ok(("", "FS")));
        assert_eq!(paper_type("Edit"), Ok(("", "Edit")));
        assert_eq!(paper_type("SD"), Ok(("", "SD")));
        assert_eq!(
            paper_type("WG"),
            Err(nom::Err::Error(VerboseError {
                errors: vec![("WG", Nom(Tag)), ("WG", Nom(Alt))]
            }))
        );
    }

    #[test]
    fn test_paper_number() {
        assert_eq!(paper_number("1488"), Ok(("", "1488")));
        assert_eq!(
            paper_number(""),
            Err(nom::Err::Error(VerboseError {
                errors: vec![("", Nom(TakeWhile1))]
            }))
        );
        assert_eq!(
            paper_number("p1488"),
            Err(nom::Err::Error(VerboseError {
                errors: vec![("p1488", Nom(TakeWhile1))]
            }))
        );
    }

    #[test]
    fn test_revision() {
        assert_eq!(revision("R"), Ok(("", "R")));
        assert_eq!(revision("r"), Ok(("", "r")));
        assert_eq!(
            revision("A"),
            Err(nom::Err::Error(VerboseError {
                errors: vec![("A", Nom(Tag))]
            }))
        );
    }

    #[test]
    fn test_revision_number() {
        assert_eq!(revision_number("0"), Ok(("", "0")));
        assert_eq!(revision_number("10"), Ok(("", "10")));
        assert_eq!(
            revision_number(""),
            Err(nom::Err::Error(VerboseError {
                errors: vec![("", Nom(TakeWhile1))]
            }))
        );
    }

    #[test]
    fn test_paper() {
        assert_eq!(
            paper("p1488"),
            Ok((
                "",
                ImplicitPaperSearchRequest {
                    paper_type: "p".to_string(),
                    paper_number: "1488".to_string(),
                    revision_number: None
                }
            ))
        );
        assert_eq!(
            paper("p1488R0"),
            Ok((
                "",
                ImplicitPaperSearchRequest {
                    paper_type: "p".to_string(),
                    paper_number: "1488".to_string(),
                    revision_number: Some(0)
                }
            ))
        );
        assert_eq!(
            paper("p 1488R0"),
            Err(nom::Err::Error(VerboseError {
                errors: vec![(" 1488R0", Nom(TakeWhile1))]
            }))
        );
    }

    #[test]
    fn test_paper_request() {
        assert_eq!(
            paper_request("[p1488]"),
            Ok((
                "",
                ImplicitPaperSearchRequest {
                    paper_type: "p".to_string(),
                    paper_number: "1488".to_string(),
                    revision_number: None
                }
            ))
        );

        assert_eq!(
            paper_request("[p1488>"),
            Ok((
                "",
                ImplicitPaperSearchRequest {
                    paper_type: "p".to_string(),
                    paper_number: "1488".to_string(),
                    revision_number: None
                }
            ))
        );

        assert_eq!(
            paper_request("[]"),
            Err(nom::Err::Error(VerboseError {
                errors: vec![("]", Nom(Tag)), ("]", Nom(Alt))]
            }))
        );
    }

    #[test]
    fn test_paper_request_with_leading_trash() {
        assert_eq!(
            paper_request_with_leading_trash("some text [p1488]"),
            Ok((
                "",
                ImplicitPaperSearchRequest {
                    paper_type: "p".to_string(),
                    paper_number: "1488".to_string(),
                    revision_number: None
                }
            ))
        );

        assert_eq!(
            paper_request_with_leading_trash("[p1488]"),
            Ok((
                "",
                ImplicitPaperSearchRequest {
                    paper_type: "p".to_string(),
                    paper_number: "1488".to_string(),
                    revision_number: None
                }
            ))
        );

        assert_eq!(
            paper_request_with_leading_trash("[p1488] some text"),
            Ok((
                " some text",
                ImplicitPaperSearchRequest {
                    paper_type: "p".to_string(),
                    paper_number: "1488".to_string(),
                    revision_number: None
                }
            ))
        );
    }

    #[test]
    fn test_many_paper_requests() {
        assert_eq!(
            many_paper_requests("some_text [p1488] and [p2000R10] ."),
            Ok((
                " .",
                vec![
                    ImplicitPaperSearchRequest {
                        paper_type: "p".to_string(),
                        paper_number: "1488".to_string(),
                        revision_number: None
                    },
                    ImplicitPaperSearchRequest {
                        paper_type: "p".to_string(),
                        paper_number: "2000".to_string(),
                        revision_number: Some(10)
                    }
                ]
            ))
        );

        assert_eq!(many_paper_requests("some_text"), Ok(("some_text", vec![])));
    }
}
