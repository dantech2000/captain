use crate::model::PullProgress;

/// Sums up the messages of one image push into a layer count and a status line.
/// The engine's push messages carry no layer ID that Captain can read, so each
/// `Preparing` message counts as one layer, and each finish message as one done.
#[derive(Debug, Clone)]
pub struct PushTracker {
    reference: String,
    status: String,
    layers: usize,
    done: usize,
    finished: bool,
}

impl PushTracker {
    pub fn new(reference: impl Into<String>) -> Self {
        Self {
            reference: reference.into(),
            status: "Starting push".into(),
            layers: 0,
            done: 0,
            finished: false,
        }
    }

    /// What is being pushed, for example `ghcr.io/team/app:1.0`.
    pub fn reference(&self) -> &str {
        &self.reference
    }

    /// The latest message that is not about one layer, for example the digest line.
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Adds one message from the push stream.
    pub fn apply(&mut self, message: &PullProgress) {
        let status = message.status.as_str();
        match status {
            "Preparing" => self.layers += 1,
            "Waiting" | "Pushing" => {}
            "Pushed" | "Layer already exists" => self.done += 1,
            _ if status.starts_with("Mounted from") => self.done += 1,
            "" => {}
            _ => self.status = status.to_string(),
        }
    }

    /// Marks the push as done when its stream ends without an error.
    pub fn finish(&mut self) {
        self.finished = true;
        self.done = self.layers;
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Overall progress from 0.0 to 1.0: the share of finished layers.
    pub fn fraction(&self) -> f32 {
        match (self.finished, self.layers) {
            (true, _) => 1.0,
            (false, 0) => 0.0,
            (false, layers) => self.done.min(layers) as f32 / layers as f32,
        }
    }

    /// Finished layers and all known layers.
    pub fn layer_counts(&self) -> (usize, usize) {
        (self.done.min(self.layers), self.layers)
    }
}

#[cfg(test)]
mod tests;
