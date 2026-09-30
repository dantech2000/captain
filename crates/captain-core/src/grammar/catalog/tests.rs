use crate::grammar::Verb;
use crate::grammar::fixture::catalog;

#[test]
fn candidates_use_the_shortest_name_that_is_not_ambiguous() {
    let texts: Vec<String> = catalog()
        .candidates(Verb::Restart.slots()[0])
        .into_iter()
        .map(|(text, _)| text)
        .collect();
    // Services first, then containers (running first), then projects. Pod
    // containers are left out.
    assert_eq!(
        texts,
        [
            "web",
            "shop/api",
            "worker",
            "postgres",
            "blog/api",
            "shop-web-1",
            "shop-api-1",
            "shop-worker-1",
            "shop-worker-2",
            "shop-postgres-1",
            "blog-api-1",
            "redis",
            "blog",
            "shop",
            "blog",
        ]
    );
    let kube: Vec<String> = catalog()
        .candidates(Verb::Forward.slots()[0])
        .into_iter()
        .map(|(text, _)| text)
        .collect();
    assert_eq!(
        kube,
        [
            "svc/default/web",
            "svc/api:80",
            "svc/api:443",
            "svc/other/web"
        ]
    );
}
