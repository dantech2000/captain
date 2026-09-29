use super::*;

#[test]
fn messages_count_and_list_failures() {
    let outcome = BulkOutcome {
        done: vec!["a".into(), "b".into(), "c".into()],
        failed: vec![("d".into(), EngineError::Api("in use".into()))],
    };
    assert_eq!(
        outcome.done_message("Deleted", "volume"),
        "Deleted 3 volumes."
    );
    assert_eq!(
        outcome.failed_title("Delete", "volume"),
        "Could not delete 1 of 4 volumes"
    );
    assert_eq!(outcome.failed_lines(), "d: engine error: in use");
}
