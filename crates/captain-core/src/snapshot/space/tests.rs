use super::{STEP_SPACE, check_space};
use crate::GIB;

#[test]
fn a_clone_needs_the_step_space_and_a_copy_also_the_disk() {
    let disk = 10 * GIB;
    assert!(check_space(Some(STEP_SPACE), disk, true).is_ok());
    assert!(check_space(Some(STEP_SPACE - 1), disk, true).is_err());
    assert!(check_space(Some(STEP_SPACE + disk - 1), disk, false).is_err());
    assert!(check_space(Some(STEP_SPACE + disk), disk, false).is_ok());
    assert!(check_space(None, disk, false).is_ok());
}
