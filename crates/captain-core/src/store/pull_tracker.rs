use crate::model::PullProgress;

/// The share of a layer's progress bar that the download fills. Extraction fills the rest.
const DOWNLOAD_SHARE: f32 = 0.8;

/// Sums up the messages of one image pull into an overall fraction and a status line.
#[derive(Debug, Clone)]
pub struct PullTracker {
    reference: String,
    status: String,
    layers: Vec<Layer>,
    finished: bool,
}

#[derive(Debug, Clone)]
struct Layer {
    id: String,
    fraction: f32,
}

impl PullTracker {
    pub fn new(reference: impl Into<String>) -> Self {
        Self {
            reference: reference.into(),
            status: "Starting pull".into(),
            layers: Vec::new(),
            finished: false,
        }
    }

    /// What is being pulled, for example `busybox:latest`.
    pub fn reference(&self) -> &str {
        &self.reference
    }

    /// The latest message that is not about one layer, for example `Pulling from library/nginx`.
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Adds one message from the pull stream.
    pub fn apply(&mut self, message: &PullProgress) {
        let (Some(id), Some(fraction)) = (&message.layer, layer_fraction(message)) else {
            if !message.status.is_empty() {
                self.status = message.status.clone();
            }
            return;
        };
        match self.layers.iter_mut().find(|layer| &layer.id == id) {
            // A layer never moves backwards, even if messages arrive out of order.
            Some(layer) => layer.fraction = layer.fraction.max(fraction),
            None => self.layers.push(Layer {
                id: id.clone(),
                fraction,
            }),
        }
    }

    /// Marks the pull as done when its stream ends without an error. Some layers,
    /// such as the image config, never report `Pull complete`.
    pub fn finish(&mut self) {
        self.finished = true;
        for layer in &mut self.layers {
            layer.fraction = 1.0;
        }
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Overall progress from 0.0 to 1.0: the mean of every known layer's progress.
    pub fn fraction(&self) -> f32 {
        if self.finished {
            return 1.0;
        }
        if self.layers.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.layers.iter().map(|layer| layer.fraction).sum();
        sum / self.layers.len() as f32
    }

    /// Finished layers and all known layers.
    pub fn layer_counts(&self) -> (usize, usize) {
        let done = self.layers.iter().filter(|l| l.fraction >= 1.0).count();
        (done, self.layers.len())
    }
}

/// How far along one layer is after `message`, or `None` if it is not a layer message.
fn layer_fraction(message: &PullProgress) -> Option<f32> {
    let bytes = match (message.current, message.total) {
        (Some(current), Some(total)) if total > 0 => (current as f32 / total as f32).min(1.0),
        _ => 0.0,
    };
    let fraction = match message.status.as_str() {
        "Pulling fs layer" | "Waiting" => 0.0,
        "Downloading" => DOWNLOAD_SHARE * bytes,
        "Verifying Checksum" | "Download complete" => DOWNLOAD_SHARE,
        "Extracting" => DOWNLOAD_SHARE + (1.0 - DOWNLOAD_SHARE) * bytes,
        "Pull complete" | "Already exists" => 1.0,
        _ => return None,
    };
    Some(fraction)
}

#[cfg(test)]
mod tests;
