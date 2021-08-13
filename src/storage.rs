use std::ops::AddAssign;

use serde::Deserialize;

use crate::utils;
use std::collections::HashMap;

#[derive(Clone, Deserialize)]
pub struct Paper {
    number: Option<String>,
    title: Option<String>,
    link: Option<String>,
    author: Option<String>,
    date: Option<String>,
    github_url: Option<String>,
}

impl Paper {
    pub fn format_with_markdownv2(&self) -> String {
        let mut new_title = String::new();

        if let Some(number) = &self.number {
            new_title = number.clone();
        }

        if let Some(source_title) = &self.title {
            if !new_title.is_empty() {
                new_title = format!("{}: {}", new_title, source_title);
            } else {
                new_title = source_title.clone();
            }
        }

        // In case if number and title are both empty - just fill with some placeholder
        if new_title.is_empty() {
            new_title = "Here should be a paper title".to_string();
        }

        let mut result: String;

        match &self.link {
            Some(link) => {
                result = format!(
                    "[{}]({})",
                    utils::markdown_v2_escape(new_title.as_str()),
                    utils::markdown_v2_escape_inline_uri(link)
                )
            }
            None => result = new_title,
        }

        if let Some(author) = &self.author {
            result.add_assign(
                format!(r#" \(by {}\)"#, utils::markdown_v2_escape(author.as_str())).as_str(),
            );
        }

        if let Some(date) = &self.date {
            result.add_assign(
                format!(r#" \({}\)"#, utils::markdown_v2_escape(date.as_str())).as_str(),
            );
        }

        if let Some(github_url) = &self.github_url {
            result.add_assign(
                format!(
                    r#" \(Related: [GitHub issue]({})\)"#,
                    utils::markdown_v2_escape(github_url.as_str())
                )
                .as_str(),
            );
        }

        return result;
    }
}

pub struct PaperDatabase {
    database: HashMap<String, Paper>,
}

impl PaperDatabase {
    pub fn new_empty() -> Self {
        PaperDatabase {
            database: HashMap::<String, Paper>::new(),
        }
    }

    pub fn new(mut initial_values: HashMap<String, Paper>) -> Self {
        for (key, value) in initial_values.iter_mut() {
            value.number = Option::from(key.clone());
        }

        PaperDatabase {
            database: initial_values,
        }
    }

    pub fn len(&self) -> usize {
        return self.database.len();
    }

    pub fn search_by_number(&self, pattern: &String, limit: u8) -> (bool, Vec<Paper>) {
        let mut result: Vec<Paper> = Vec::new();
        let mut is_limit_reached = false;

        let re = regex::RegexBuilder::new(pattern)
            .case_insensitive(true)
            .build()
            .unwrap();

        for (_, paper) in self.database.iter() {
            if result.len() == limit as usize {
                is_limit_reached = true;
                break;
            }

            if paper.number.is_some() && re.is_match(paper.number.as_ref().unwrap().as_str()) {
                result.push(paper.clone());
            }
        }

        return (is_limit_reached, result);
    }

    pub fn search_any(&self, pattern: &String, limit: u8) -> (bool, Vec<Paper>) {
        let mut result: Vec<Paper> = Vec::new();
        let mut is_limit_reached = false;

        let re = regex::RegexBuilder::new(pattern)
            .case_insensitive(true)
            .build()
            .unwrap();

        for (_, paper) in self.database.iter() {
            if result.len() == limit as usize {
                is_limit_reached = true;
                break;
            }

            if (paper.number.is_some() && re.is_match(paper.number.as_ref().unwrap().as_str()))
                || (paper.title.is_some() && re.is_match(paper.title.as_ref().unwrap().as_str()))
                || (paper.author.is_some() && re.is_match(paper.author.as_ref().unwrap().as_str()))
            {
                result.push(paper.clone());
            }
        }

        return (is_limit_reached, result);
    }
}

pub type PapersStorage = std::sync::Arc<std::sync::Mutex<crate::storage::PaperDatabase>>;
