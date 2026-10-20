use serde::Deserialize;
use stagezero_core::DomainError;
use stagezero_core::entities::{ApplicationStatus, HuntCriteria, TeamSizeRange};
use stagezero_core::pagination::{
    ApplicationQuery, ApplicationSort, ExploreQuery, PageRequest, SortDirection, StartupSort,
};

fn non_blank(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.trim().is_empty())
}

pub fn parse_team_size(size: Option<&str>) -> Result<TeamSizeRange, DomainError> {
    match size.map(str::trim).filter(|size| !size.is_empty()) {
        Some(size) => size.parse::<TeamSizeRange>(),
        None => Ok(TeamSizeRange::default()),
    }
}

fn parse_direction(
    order: Option<&str>,
    fallback: SortDirection,
) -> Result<SortDirection, DomainError> {
    match order.map(str::trim).map(str::to_lowercase).as_deref() {
        None | Some("") => Ok(fallback),
        Some("asc") => Ok(SortDirection::Asc),
        Some("desc") => Ok(SortDirection::Desc),
        Some(other) => Err(DomainError::InvalidInput(format!(
            "order must be `asc` or `desc`, got `{other}`"
        ))),
    }
}

#[derive(Debug, Deserialize)]
pub struct ExploreParams {
    pub batch: String,
    pub size: Option<String>,
    pub industry: Option<String>,
    pub q: Option<String>,
    pub hide_contacted: Option<bool>,
    pub sort: Option<String>,
    pub order: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

impl ExploreParams {
    pub fn into_parts(self) -> Result<(HuntCriteria, ExploreQuery), DomainError> {
        let sort = match self.sort.as_deref().map(str::trim) {
            None | Some("") => None,
            Some("name") => Some(StartupSort::Name),
            Some("team_size") => Some(StartupSort::TeamSize),
            Some(other) => {
                return Err(DomainError::InvalidInput(format!(
                    "sort must be `name` or `team_size`, got `{other}`"
                )));
            }
        };
        let criteria = HuntCriteria {
            team_size: parse_team_size(self.size.as_deref())?,
            batch: self.batch,
            industry: non_blank(self.industry),
            limit: None,
            dry_run: true,
        };
        let query = ExploreQuery {
            search: non_blank(self.q),
            hide_contacted: self.hide_contacted.unwrap_or(false),
            sort,
            direction: parse_direction(self.order.as_deref(), SortDirection::Asc)?,
            page: PageRequest::new(self.page, self.per_page),
        };
        Ok((criteria, query))
    }
}

#[derive(Debug, Deserialize)]
pub struct ApplicationParams {
    pub status: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

impl ApplicationParams {
    pub fn into_query(self) -> Result<ApplicationQuery, DomainError> {
        let status = non_blank(self.status)
            .map(|status| status.parse::<ApplicationStatus>())
            .transpose()?;
        let sort = match self.sort.as_deref().map(str::trim) {
            None | Some("") | Some("applied_at") => ApplicationSort::AppliedAt,
            Some("company_name") => ApplicationSort::CompanyName,
            Some("status") => ApplicationSort::Status,
            Some(other) => {
                return Err(DomainError::InvalidInput(format!(
                    "sort must be `applied_at`, `company_name` or `status`, got `{other}`"
                )));
            }
        };
        Ok(ApplicationQuery {
            status,
            search: non_blank(self.q),
            sort,
            direction: parse_direction(self.order.as_deref(), SortDirection::Desc)?,
            page: PageRequest::new(self.page, self.per_page),
        })
    }
}
