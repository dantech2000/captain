use gpui_kit::AssetSource;

use super::CaptainAssets;
use crate::icons::CaptainIcon;

#[test]
fn every_icon_file_and_lucide_load_through_the_asset_source() {
    for icon in CaptainIcon::ALL {
        let paths = [icon.line_path(), icon.fill_path()]
            .into_iter()
            .chain(icon.second_fill_path());
        for path in paths {
            let bytes = CaptainAssets.load(path).unwrap().expect(path);
            assert!(bytes.starts_with(b"<svg"), "{path}");
        }
    }
    let lucide = gpui_kit::assets::IconName::Search.path();
    assert!(CaptainAssets.load(&lucide).unwrap().is_some());
}
