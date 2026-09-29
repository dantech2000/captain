use std::sync::{Arc, OnceLock};

use gpui_kit::*;

/// The Glass helm app icon, rendered by scripts/build-icons.sh.
static BRAND_PNG: &[u8] = include_bytes!("../../../../assets/icon/brand.png");

/// The Captain app icon at `size`. The artwork has the macOS icon margin built in,
/// so the visible squircle is about 80% of `size`.
pub fn brand_mark(size: Pixels) -> Img {
    static IMAGE: OnceLock<Arc<Image>> = OnceLock::new();
    let image = IMAGE
        .get_or_init(|| Arc::new(Image::from_bytes(ImageFormat::Png, BRAND_PNG.to_vec())))
        .clone();
    img(image).size(size).flex_shrink_0()
}
