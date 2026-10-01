use std::path::Path;

use super::tab_title;

#[test]
fn shell_title_then_folder_name_then_tilde() {
    let home = Path::new("/home/me");
    let shop = home.join("code").join("shop");
    assert_eq!(tab_title(Some("vim".into()), &shop, Some(home)), "vim");
    assert_eq!(tab_title(Some(" ".into()), &shop, Some(home)), "shop");
    assert_eq!(tab_title(None, home, Some(home)), "~");
}
