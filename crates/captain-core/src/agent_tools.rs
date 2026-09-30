//! What Captain's MCP server (`captain mcp`) tells agents, without the protocol:
//! the answers of the tools, name checks, log caps, secret masking, the
//! delimiters around untrusted container output, the `agent_tools` settings, and
//! the activity log. See docs/features/0038-agent-tools.md.

mod action_report;
mod activity;
mod container_rows;
mod disk_report;
mod engine_report;
mod help;
mod inspect_report;
mod log_buffer;
mod log_query;
mod log_report;
mod mask;
mod problem_report;
mod project_rows;
mod readiness;
mod settings;
mod target;
mod untrusted;

#[cfg(test)]
mod test_fleet;

pub use action_report::{ActionReport, TaskReport, memory_raise, raised_sentence, task_report};
pub use activity::{
    ACTIVITY_CAP, ACTIVITY_FILE, Activity, activity_path, append_activity, read_activity,
};
pub use container_rows::{ContainerList, ContainerRow, container_list};
pub use disk_report::{CleanupGroup, DiskCategory, DiskItem, DiskReport, disk_report};
pub use engine_report::{EngineAnswer, EngineReport, engine_report};
pub use help::{GUIDE, HelpReport, ToolLine};
pub use inspect_report::{EnvEntry, InspectReport, inspect_report};
pub use log_buffer::LogBuffer;
pub use log_query::{
    DEFAULT_TAIL, FILTERED_TAIL, LogQuery, MAX_BYTES, MAX_LINE_CHARS, MAX_LINES, MAX_TAIL,
    parse_since,
};
pub use log_report::{LogReport, SourcedLine, log_report};
pub use mask::{MASK, mask_secrets};
pub use problem_report::{ContainerProblem, ProblemReport, problem_report};
pub use project_rows::{ProjectList, ProjectRow, project_list};
pub use readiness::{Readiness, WaitReport, readiness, status};
pub use settings::{AgentAction, AgentToolsSettings};
pub use target::{check_name, find_container, find_project, find_service};
pub use untrusted::{UNTRUSTED_LABEL, clean, plain, wrap_untrusted};
