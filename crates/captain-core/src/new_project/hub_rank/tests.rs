use super::*;

fn repo(name: &str, official: bool, pulls: u64, stars: u64) -> HubRepo {
    HubRepo {
        name: name.into(),
        description: String::new(),
        stars,
        pulls,
        official,
    }
}

#[test]
fn name_matches_come_first_then_official_then_pulls_then_stars() {
    let repos = vec![
        repo("someone/uptime-monitor-tool", false, 900_000_000, 0),
        repo("audius/uptime", false, 149_000, 0),
        repo("mijovi/uptime", false, 23_000, 5),
        repo("tiny/uptime", false, 23_000, 1),
        repo("louislam/uptime-kuma", false, 100_000_000, 4_000),
        repo("myuptime/other", false, 5_000_000, 0),
        repo("uptime", true, 10, 0),
    ];

    let names: Vec<String> = rank_search(" Uptime ", repos)
        .into_iter()
        .map(|repo| repo.name)
        .collect();

    assert_eq!(
        names,
        [
            "uptime",
            "someone/uptime-monitor-tool",
            "louislam/uptime-kuma",
            "audius/uptime",
            "mijovi/uptime",
            "tiny/uptime",
            "myuptime/other",
        ]
    );
}
