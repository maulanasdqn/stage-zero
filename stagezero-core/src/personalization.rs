use serde::{Deserialize, Serialize};

use crate::entities::Startup;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Highlight {
    pub text: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub pinned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighlightMatch {
    pub text: String,
    pub score: u32,
    pub matched: Vec<String>,
    pub pinned: bool,
}

const CATEGORY_WEIGHT: u32 = 3;
const ONE_LINER_WEIGHT: u32 = 2;
const DESCRIPTION_WEIGHT: u32 = 1;

fn is_word_character(character: char) -> bool {
    character.is_alphanumeric()
}

pub fn contains_term(haystack: &str, term: &str) -> bool {
    let term = term.trim().to_lowercase();
    if term.is_empty() {
        return false;
    }
    let haystack = haystack.to_lowercase();
    haystack.match_indices(&term).any(|(start, matched)| {
        let before_ok = haystack[..start]
            .chars()
            .next_back()
            .is_none_or(|character| !is_word_character(character));
        let rest = &haystack[start + matched.len()..];
        let rest = rest
            .strip_prefix("es")
            .or_else(|| rest.strip_prefix('s'))
            .unwrap_or(rest);
        let after_ok = rest
            .chars()
            .next()
            .is_none_or(|character| !is_word_character(character));
        before_ok && after_ok
    })
}

fn score_highlight(highlight: &Highlight, startup: &Startup) -> HighlightMatch {
    let categories = startup
        .industries
        .iter()
        .chain(startup.tags.iter())
        .cloned()
        .collect::<Vec<_>>()
        .join(" | ");
    let mut score = 0;
    let mut matched = Vec::new();
    for keyword in &highlight.keywords {
        let weight = [
            (categories.as_str(), CATEGORY_WEIGHT),
            (startup.one_liner.as_str(), ONE_LINER_WEIGHT),
            (startup.description.as_str(), DESCRIPTION_WEIGHT),
        ]
        .into_iter()
        .filter(|(field, _)| contains_term(field, keyword))
        .map(|(_, weight)| weight)
        .max();
        if let Some(weight) = weight {
            score += weight;
            matched.push(keyword.trim().to_string());
        }
    }
    HighlightMatch {
        text: highlight.text.trim().to_string(),
        score,
        matched,
        pinned: highlight.pinned,
    }
}

pub fn select_highlights(
    highlights: &[Highlight],
    startup: &Startup,
    limit: usize,
) -> Vec<HighlightMatch> {
    let scored = highlights
        .iter()
        .filter(|highlight| !highlight.text.trim().is_empty())
        .map(|highlight| score_highlight(highlight, startup))
        .collect::<Vec<_>>();

    let mut relevant = scored
        .iter()
        .enumerate()
        .filter(|(_, candidate)| !candidate.pinned && candidate.score > 0)
        .collect::<Vec<_>>();
    relevant.sort_by(|(left_index, left), (right_index, right)| {
        right
            .score
            .cmp(&left.score)
            .then(left_index.cmp(right_index))
    });

    let pinned = scored
        .iter()
        .enumerate()
        .filter(|(_, candidate)| candidate.pinned);
    let fallback = scored
        .iter()
        .enumerate()
        .filter(|(_, candidate)| !candidate.pinned && candidate.score == 0);

    pinned
        .chain(relevant)
        .chain(fallback)
        .take(limit)
        .map(|(_, candidate)| candidate.clone())
        .collect()
}

pub fn render_bullets(selected: &[HighlightMatch]) -> String {
    selected
        .iter()
        .map(|highlight| format!("- {}", highlight.text))
        .collect::<Vec<_>>()
        .join("\n")
}
