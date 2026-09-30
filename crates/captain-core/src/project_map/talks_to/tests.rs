use super::{host_tokens, talks_to};
use crate::model::EnvVar;
use crate::project_map::MapService;

fn service(id: &str, env: &[&str]) -> MapService {
    MapService {
        id: id.into(),
        name: id.into(),
        container_name: format!("shop-{id}-1"),
        env: env.iter().map(|e| EnvVar::parse(e)).collect(),
        ..MapService::default()
    }
}

#[test]
fn reads_hosts_from_urls_host_port_and_host_keys() {
    let tokens = |entry: &str| host_tokens(&EnvVar::parse(entry));
    assert_eq!(
        tokens("DATABASE_URL=postgres://app:s3cret@postgres:5432/shop?sslmode=disable"),
        ["postgres"]
    );
    assert_eq!(
        tokens("BROKERS=kafka:9092,kafka-2:9092"),
        ["kafka", "kafka-2"]
    );
    assert_eq!(tokens("REDIS_HOST=redis"), ["redis"]);
    // A bare word counts only under a host key, and a colon needs a numeric port.
    assert!(tokens("POSTGRES_USER=postgres").is_empty());
    assert!(tokens("LOG_FORMAT=json:pretty").is_empty());
    assert!(tokens("CACHE_HOST=").is_empty());
}

#[test]
fn links_to_other_services_by_service_or_container_name() {
    let all = vec![
        service(
            "api",
            &[
                "DATABASE_URL=postgres://app@postgres/shop",
                "CACHE=shop-redis-1:6379",
            ],
        ),
        service("postgres", &["POSTGRES_USER=postgres", "PGHOST=postgres"]),
        service("redis", &[]),
        service("worker", &["API_URL=http://api:8080", "QUEUE=redis:6379"]),
    ];
    assert_eq!(talks_to(&all[0], &all), ["postgres", "redis"]);
    // A service naming itself is not an edge.
    assert!(talks_to(&all[1], &all).is_empty());
    assert_eq!(talks_to(&all[3], &all), ["api", "redis"]);
}
