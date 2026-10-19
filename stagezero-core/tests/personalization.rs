use stagezero_core::entities::{Applicant, EmailTemplate, Startup, TemplateContext};
use stagezero_core::personalization::{Highlight, contains_term, select_highlights};

fn highlight(text: &str, keywords: &[&str], pinned: bool) -> Highlight {
    Highlight {
        text: text.into(),
        keywords: keywords.iter().map(|keyword| keyword.to_string()).collect(),
        pinned,
    }
}

fn library() -> Vec<Highlight> {
    vec![
        highlight("scraping", &["scraping", "data", "infrastructure"], false),
        highlight(
            "workflows",
            &["legal", "compliance", "documents", "workflow"],
            false,
        ),
        highlight("agents", &["ai", "agent", "llm"], false),
        highlight("edtech", &["education", "students"], false),
        highlight("community", &["community"], false),
    ]
}

fn startup(industries: &[&str], tags: &[&str], one_liner: &str, description: &str) -> Startup {
    Startup {
        slug: "neomatter".into(),
        name: "Neomatter".into(),
        website: None,
        batch: "Fall 2026".into(),
        team_size: Some(2),
        industries: industries.iter().map(|value| value.to_string()).collect(),
        one_liner: one_liner.into(),
        tags: tags.iter().map(|value| value.to_string()).collect(),
        description: description.into(),
    }
}

#[test]
fn terms_match_on_word_boundaries_and_plurals() {
    assert!(contains_term(
        "AI that works for in-house legal teams",
        "ai"
    ));
    assert!(contains_term(
        "Next-generation agent operating system",
        "agent"
    ));
    assert!(contains_term("autonomous agents for ops", "agent"));
    assert!(contains_term("B2B | Legal", "legal"));
    assert!(!contains_term("email marketing", "ai"));
    assert!(!contains_term("paralegal software", "legal"));
    assert!(!contains_term("anything", "  "));
}

#[test]
fn most_relevant_highlights_win_for_each_startup() {
    let legal = startup(
        &["B2B", "Legal"],
        &["Legal", "Artificial Intelligence"],
        "AI that works for in-house legal teams",
        "We automate contract review and compliance documents.",
    );
    let picked = select_highlights(&library(), &legal, 3);
    let texts = picked
        .iter()
        .map(|item| item.text.as_str())
        .collect::<Vec<_>>();
    assert_eq!(texts, vec!["workflows", "agents", "scraping"]);
    assert_eq!(picked[0].matched, vec!["legal", "compliance", "documents"]);
    assert_eq!(picked[2].score, 0);

    let edtech = startup(&["Education"], &["EdTech"], "Tutoring for students", "");
    let picked = select_highlights(&library(), &edtech, 2);
    assert_eq!(picked[0].text, "edtech");
}

#[test]
fn pinned_highlights_always_lead_and_limit_is_respected() {
    let mut highlights = library();
    highlights[4].pinned = true;
    let legal = startup(&["Legal"], &[], "Legal AI", "");
    let picked = select_highlights(&highlights, &legal, 2);
    let texts = picked
        .iter()
        .map(|item| item.text.as_str())
        .collect::<Vec<_>>();
    assert_eq!(texts, vec!["community", "workflows"]);
}

#[test]
fn highlights_variable_renders_bullets_in_template() {
    let applicant = Applicant {
        name: "Ada Lovelace".into(),
        email: "ada@example.dev".into(),
        portfolio: None,
        highlights: library(),
        max_highlights: 2,
    };
    let template = EmailTemplate {
        subject: "s".into(),
        body: "Proof:\n{{highlights}}".into(),
    };
    let target = startup(&["Education"], &[], "AI tutors for students", "");
    let context = TemplateContext {
        startup: &target,
        founder: None,
        applicant: &applicant,
    };
    assert_eq!(template.render(&context).1, "Proof:\n- edtech\n- agents");
}
