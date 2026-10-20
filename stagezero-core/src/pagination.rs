use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::entities::{ApplicationStatus, DirectoryEntry};

pub const DEFAULT_PER_PAGE: usize = 20;
pub const MAX_PER_PAGE: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageRequest {
    pub page: usize,
    pub per_page: usize,
}

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: DEFAULT_PER_PAGE,
        }
    }
}

impl PageRequest {
    pub fn new(page: Option<usize>, per_page: Option<usize>) -> Self {
        Self {
            page: page.unwrap_or(1).max(1),
            per_page: per_page.unwrap_or(DEFAULT_PER_PAGE).clamp(1, MAX_PER_PAGE),
        }
    }

    pub fn offset(&self) -> usize {
        (self.page - 1) * self.per_page
    }

    pub fn meta(&self, total: usize) -> PageMeta {
        PageMeta {
            page: self.page,
            per_page: self.per_page,
            total,
            total_pages: total.div_ceil(self.per_page).max(1),
        }
    }

    pub fn slice<T>(&self, items: Vec<T>) -> Paginated<T> {
        let pagination = self.meta(items.len());
        let data = items
            .into_iter()
            .skip(self.offset())
            .take(self.per_page)
            .collect();
        Paginated { data, pagination }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageMeta {
    pub page: usize,
    pub per_page: usize,
    pub total: usize,
    pub total_pages: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub pagination: PageMeta,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

impl SortDirection {
    pub fn apply(&self, ordering: Ordering) -> Ordering {
        match self {
            SortDirection::Asc => ordering,
            SortDirection::Desc => ordering.reverse(),
        }
    }

    pub fn as_sql(&self) -> &'static str {
        match self {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StartupSort {
    Name,
    TeamSize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExploreQuery {
    pub search: Option<String>,
    pub hide_contacted: bool,
    pub sort: Option<StartupSort>,
    pub direction: SortDirection,
    pub page: PageRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExploreResult {
    pub data: Vec<DirectoryEntry>,
    pub pagination: PageMeta,
    pub batch_total: usize,
    pub tracked: usize,
}

pub fn matches_search(entry: &DirectoryEntry, needle: &str) -> bool {
    let startup = &entry.startup;
    [&startup.name, &startup.one_liner, &startup.slug]
        .into_iter()
        .chain(startup.industries.iter())
        .chain(startup.tags.iter())
        .any(|field| field.to_lowercase().contains(needle))
}

pub fn compare_entries(
    left: &DirectoryEntry,
    right: &DirectoryEntry,
    sort: StartupSort,
) -> Ordering {
    match sort {
        StartupSort::Name => left
            .startup
            .name
            .to_lowercase()
            .cmp(&right.startup.name.to_lowercase()),
        StartupSort::TeamSize => left.startup.team_size.cmp(&right.startup.team_size),
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationSort {
    #[default]
    AppliedAt,
    CompanyName,
    Status,
}

impl ApplicationSort {
    pub fn column(&self) -> &'static str {
        match self {
            ApplicationSort::AppliedAt => "applied_at",
            ApplicationSort::CompanyName => "company_name COLLATE NOCASE",
            ApplicationSort::Status => "status",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplicationQuery {
    pub status: Option<ApplicationStatus>,
    pub search: Option<String>,
    pub sort: ApplicationSort,
    pub direction: SortDirection,
    pub page: PageRequest,
}

impl Default for ApplicationQuery {
    fn default() -> Self {
        Self {
            status: None,
            search: None,
            sort: ApplicationSort::AppliedAt,
            direction: SortDirection::Desc,
            page: PageRequest::default(),
        }
    }
}
