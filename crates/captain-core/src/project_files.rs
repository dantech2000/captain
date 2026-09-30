//! The project editor's pure side: which files a project may edit, safe reads and
//! saves, the YAML outline for errors and completion, the vendored Compose schema,
//! and the readers for `docker compose config`, `docker build --check`, and
//! `up --dry-run` output. See docs/features/0039-compose-and-dockerfile-editor.md.

mod build_check;
mod compose_check;
mod disk;
mod dry_run;
mod line_problem;
mod listing;
mod schema;
mod yaml_outline;

pub use build_check::build_check_problems;
pub use compose_check::compose_problems;
pub use disk::{LoadedText, SaveError, TextVersion, disk_version, read_text, save_text};
pub use dry_run::{ChangeKind, ImageStep, ServiceChange, UpPreview, parse_dry_run};
pub use line_problem::{LineProblem, Severity};
pub use listing::{EditableFile, FileKind, compose_files, dockerfiles, is_inside};
pub use schema::{ComposeSchema, SchemaKey};
pub use yaml_outline::{CursorContext, cursor_context, key_line};
