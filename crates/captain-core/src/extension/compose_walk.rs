//! The files to copy for a Compose file at an extension image's root, found file
//! by file: a Compose file that the root one includes or extends can name more
//! files, relative to its own folder. See
//! <https://docs.docker.com/reference/compose-file/include/>.

use std::collections::BTreeSet;

use super::compose_refs::{compose_files, compose_references};

/// How deep includes may nest below the root Compose file.
const MAX_DEPTH: usize = 8;

/// Walks the Compose files from the root one. Each path is relative to the image
/// root and stays inside it; every file comes back once, so a cycle ends.
pub struct ComposeWalk {
    pending: Vec<(String, usize)>,
    seen: BTreeSet<String>,
}

impl ComposeWalk {
    /// Starts at `root`, the Compose file, which the caller has copied already.
    pub fn new(root: &str) -> Self {
        Self {
            pending: vec![(root.to_string(), 0)],
            seen: BTreeSet::from([root.to_string()]),
        }
    }

    /// The next copied Compose file to read, and its depth below the root.
    pub fn next_file(&mut self) -> Option<(String, usize)> {
        self.pending.pop()
    }

    /// The files that `file`, at `depth`, names and that no earlier file named.
    /// The caller copies them; the Compose files among them come back from
    /// `next_file`, down to `MAX_DEPTH`.
    pub fn read(&mut self, file: &str, depth: usize, yaml: &str) -> Vec<String> {
        let folder = match file.rsplit_once('/') {
            Some((folder, _)) => format!("{folder}/"),
            None => String::new(),
        };
        let fresh: Vec<String> = compose_references(yaml)
            .into_iter()
            .map(|reference| format!("{folder}{reference}"))
            .filter(|path| self.seen.insert(path.clone()))
            .collect();
        if depth < MAX_DEPTH {
            for nested in compose_files(yaml) {
                let nested = format!("{folder}{nested}");
                if fresh.contains(&nested) && !self.pending.iter().any(|(p, _)| *p == nested) {
                    self.pending.push((nested, depth + 1));
                }
            }
        }
        fresh
    }
}

#[cfg(test)]
mod tests;
