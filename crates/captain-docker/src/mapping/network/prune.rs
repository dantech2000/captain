use std::collections::HashMap;

/// The `filters` for `POST /networks/prune`: only networks with `label`, if given.
pub fn network_prune_filters(label: Option<&str>) -> HashMap<String, Vec<String>> {
    label
        .map(|label| ("label".to_string(), vec![label.to_string()]))
        .into_iter()
        .collect()
}
