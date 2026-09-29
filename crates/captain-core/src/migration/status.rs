/// How far one item of a run has got.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum StepStatus {
    #[default]
    Pending,
    /// Copying. `total` is 0 when the size is unknown.
    Running {
        done: u64,
        total: u64,
    },
    Done,
    Failed(String),
    /// Not copied, for the reason given, for example "already in the target".
    Skipped(String),
}

impl StepStatus {
    /// True once the item needs no more work: done or skipped.
    pub fn is_settled(&self) -> bool {
        matches!(self, Self::Done | Self::Skipped(_))
    }

    /// The share of the item that is copied, from 0.0 to 1.0.
    pub fn fraction(&self) -> f32 {
        match self {
            Self::Running { done, total } if *total > 0 => {
                (*done as f64 / *total as f64).min(1.0) as f32
            }
            Self::Done | Self::Skipped(_) => 1.0,
            _ => 0.0,
        }
    }
}
