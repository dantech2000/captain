//! Docker Hub search and tags over the network, without an account. Ignored by
//! default: `cargo test -p captain-docker --test live_hub -- --ignored`.

use captain_core::new_project::DockerHub;
use captain_docker::DockerHubClient;
use futures::executor::block_on;

#[test]
#[ignore = "talks to hub.docker.com"]
fn search_finds_the_official_image_first_and_its_tags() {
    let repos = block_on(DockerHubClient.search("postgres")).expect("search");
    assert_eq!(
        repos.first().map(|repo| repo.name.as_str()),
        Some("postgres")
    );
    assert!(repos[0].official && repos[0].pulls > 0);

    let tags = block_on(DockerHubClient.tags("postgres")).expect("tags");
    assert!(!tags.is_empty());
}
