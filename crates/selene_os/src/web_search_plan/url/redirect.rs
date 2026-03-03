#![forbid(unsafe_code)]

use crate::web_search_plan::url::canonical::canonicalize_url;
use crate::web_search_plan::url::UrlFetchErrorKind;
use std::collections::BTreeSet;
use url::Url;

#[derive(Debug, Clone)]
pub struct RedirectState {
    visited: BTreeSet<String>,
    depth: usize,
    max_depth: usize,
    allow_scheme_downgrade: bool,
}

impl RedirectState {
    pub fn new(
        initial_canonical_url: &str,
        max_depth: usize,
        allow_scheme_downgrade: bool,
    ) -> Self {
        let mut visited = BTreeSet::new();
        visited.insert(initial_canonical_url.to_string());
        Self {
            visited,
            depth: 0,
            max_depth,
            allow_scheme_downgrade,
        }
    }

    pub fn resolve_next(
        &mut self,
        current_url: &str,
        location_header: &str,
    ) -> Result<String, UrlFetchErrorKind> {
        if self.depth >= self.max_depth {
            return Err(UrlFetchErrorKind::RedirectDepthExceeded);
        }
        if location_header.trim().is_empty() {
            return Err(UrlFetchErrorKind::RedirectMissingLocation);
        }

        let base = Url::parse(current_url).map_err(|_| UrlFetchErrorKind::InvalidUrl)?;
        let joined = base
            .join(location_header)
            .map_err(|_| UrlFetchErrorKind::InvalidUrl)?;
        let target = canonicalize_url(joined.as_str())?;

        if !self.allow_scheme_downgrade && base.scheme() == "https" {
            let target_scheme = Url::parse(&target.canonical_url)
                .map_err(|_| UrlFetchErrorKind::InvalidUrl)?
                .scheme()
                .to_string();
            if target_scheme == "http" {
                return Err(UrlFetchErrorKind::RedirectDowngradeBlocked);
            }
        }

        if self.visited.contains(&target.canonical_url) {
            return Err(UrlFetchErrorKind::RedirectLoopDetected);
        }

        self.depth = self.depth.saturating_add(1);
        self.visited.insert(target.canonical_url.clone());
        Ok(target.canonical_url)
    }
}
