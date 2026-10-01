//! The order of Docker Hub search results. The Hub's own order puts small
//! personal repositories with the exact name above popular images, so Captain
//! sorts the page again.

use super::docker_hub::HubRepo;

/// `repos` in picker order: names that are or start with `query` first, then
/// official images, then by pulls, then by stars. The name is the part after
/// the namespace, unless `query` has a namespace too.
pub fn rank_search(query: &str, mut repos: Vec<HubRepo>) -> Vec<HubRepo> {
    let query = query.trim().to_lowercase();
    repos.sort_by_cached_key(|repo| {
        (
            !name_matches(&query, &repo.name),
            !repo.official,
            std::cmp::Reverse(repo.pulls),
            std::cmp::Reverse(repo.stars),
        )
    });
    repos
}

fn name_matches(query: &str, name: &str) -> bool {
    let name = name.to_lowercase();
    let name = match query.contains('/') {
        true => name.as_str(),
        false => name.rsplit('/').next().unwrap_or(&name),
    };
    !query.is_empty() && name.starts_with(query)
}

#[cfg(test)]
mod tests;
