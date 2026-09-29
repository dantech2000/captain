/// The processes of a running container, as `docker top` shows them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProcessTable {
    /// The column titles, for example `PID` and `CMD`.
    pub titles: Vec<String>,
    /// One row per process, with one value per title.
    pub rows: Vec<Vec<String>>,
}
