use super::*;

#[test]
fn urls_encode_the_query_and_use_library_for_official_images() {
    assert_eq!(
        search_url(" my db ", 10),
        "https://hub.docker.com/v2/search/repositories/?query=my%20db&page_size=10&ordering=-pull_count"
    );
    assert_eq!(
        tags_url("postgres", 50),
        "https://hub.docker.com/v2/namespaces/library/repositories/postgres/tags?page_size=50&ordering=last_updated"
    );
    assert!(tags_url("bitnami/redis", 50).contains("/namespaces/bitnami/repositories/redis/"));
}

#[test]
fn search_results_drop_the_library_namespace() {
    let body = r#"{"count":2,"results":[
        {"repo_name":"bitnami/postgresql","short_description":"Bitnami","star_count":10,"pull_count":5000,"is_official":false},
        {"repo_name":"postgres","short_description":null,"star_count":14000,"pull_count":1000000000,"is_official":true}
    ]}"#;

    let repos = parse_search(body).unwrap();

    assert_eq!(repos[0].name, "bitnami/postgresql");
    assert_eq!(repos[1].name, "postgres");
    assert!(repos[1].official);
    assert!(parse_search("<html>").is_err());
}

#[test]
fn a_429_waits_for_retry_after() {
    assert_eq!(
        status_error(429, Some("12")),
        HubError::RateLimited(Duration::from_secs(12))
    );
    assert_eq!(status_error(429, None), HubError::RateLimited(DEFAULT_WAIT));
    assert!(matches!(status_error(503, None), HubError::Unavailable(_)));
}

#[test]
fn counts_are_short() {
    assert_eq!(count_label(950), "950");
    assert_eq!(count_label(12_345), "12K");
    assert_eq!(count_label(3_456_789), "3.4M");
    assert_eq!(count_label(1_000_000_000), "1B");
}
