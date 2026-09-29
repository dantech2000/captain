/// The shell that classic builds wrap every step in.
const SHELL: &str = "/bin/sh -c ";
/// Classic builds mark a step that only changes metadata with this after the shell.
const NOP: &str = "#(nop)";
/// BuildKit appends this to every step it records.
const BUILDKIT: &str = "# buildkit";

/// One step of an image's history, newest first, from `docker history`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImageLayer {
    /// The image ID of this step, or `<missing>` for a step built elsewhere.
    pub id: String,
    /// Creation time as a Unix timestamp in seconds.
    pub created: i64,
    /// The Dockerfile step as the engine records it, for example
    /// `/bin/sh -c #(nop)  CMD ["sh"]`.
    pub created_by: String,
    /// The bytes this step adds. Metadata steps add none.
    pub size: u64,
}

impl ImageLayer {
    /// The step as a Dockerfile line, for example `CMD ["sh"]` or `RUN apk add curl`.
    /// It drops the shell wrapper and the BuildKit marker, and folds runs of spaces.
    pub fn command(&self) -> String {
        let raw = self.created_by.trim();
        let raw = raw.strip_suffix(BUILDKIT).unwrap_or(raw).trim_end();
        let step = match raw.find(SHELL) {
            Some(at) => {
                let rest = raw[at + SHELL.len()..].trim_start();
                match rest.strip_prefix(NOP) {
                    Some(instruction) => instruction.to_string(),
                    None => format!("RUN {rest}"),
                }
            }
            None => raw.to_string(),
        };
        step.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// This layer's size as a fraction of `largest`, from 0.0 to 1.0.
    pub fn size_fraction(&self, largest: u64) -> f32 {
        if largest == 0 {
            0.0
        } else {
            (self.size as f64 / largest as f64).min(1.0) as f32
        }
    }
}

/// The size of the largest layer, which the inspector scales the size bars to.
pub fn largest_layer_size(layers: &[ImageLayer]) -> u64 {
    layers.iter().map(|layer| layer.size).max().unwrap_or(0)
}

#[cfg(test)]
mod tests;
