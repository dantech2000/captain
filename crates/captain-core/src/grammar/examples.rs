use crate::model::ContainerState;

use super::catalog::{Catalog, Target};
use super::verb::Verb;

/// The most examples the empty palette shows.
const MAX_EXAMPLES: usize = 5;

/// Commands to try, built from live names: `restart api`, `logs worker --since 10m`,
/// `disk`, `forward svc/web 8080`, `shell postgres`.
pub fn examples(catalog: &Catalog) -> Vec<String> {
    let running: Vec<String> = catalog
        .candidates(Verb::Shell.slots()[0])
        .into_iter()
        .filter(|(_, target)| match target {
            Target::Container(id) => catalog
                .container(id)
                .is_some_and(|c| c.state == ContainerState::Running),
            _ => false,
        })
        .map(|(text, _)| text)
        .collect();
    let pick = |n: usize| running.get(n).or(running.first());

    let mut examples = Vec::new();
    if let Some(name) = pick(0) {
        examples.push(format!("restart {name}"));
    }
    if let Some(name) = pick(1) {
        examples.push(format!("logs {name} --since 10m"));
    }
    examples.push("disk".to_string());
    let kube = catalog
        .candidates(Verb::Forward.slots()[0])
        .into_iter()
        .next();
    if let Some((service, _)) = kube {
        examples.push(format!("forward {service} 8080"));
    }
    if let Some(name) = pick(2) {
        examples.push(format!("shell {name}"));
    }
    for fallback in ["go images", "go settings"] {
        if examples.len() < 4 {
            examples.push(fallback.to_string());
        }
    }
    examples.truncate(MAX_EXAMPLES);
    examples
}
