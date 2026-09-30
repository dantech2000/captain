use std::cell::RefCell;
use std::collections::HashMap;

use gpui_kit::ElementId;

thread_local! {
    /// How many live hover listeners each element with a hint has.
    static LIVE: RefCell<HashMap<ElementId, usize>> = RefCell::default();
}

/// Held by the hover listener of an element with a hint. GPUI drops an element's
/// listeners after the first frame without the element, and a removed element
/// never reports its hover-out. While one of these lives, the element is on
/// screen, so its hint may stay.
pub struct Alive(ElementId);

impl Alive {
    pub fn new(id: ElementId) -> Self {
        LIVE.with_borrow_mut(|live| *live.entry(id.clone()).or_default() += 1);
        Self(id)
    }
}

impl Drop for Alive {
    fn drop(&mut self) {
        LIVE.with_borrow_mut(|live| {
            if let Some(count) = live.get_mut(&self.0) {
                *count -= 1;
                if *count == 0 {
                    live.remove(&self.0);
                }
            }
        });
    }
}

/// True while an element with the id `id` and a hint is on screen.
pub fn is_alive(id: &ElementId) -> bool {
    LIVE.with_borrow(|live| live.contains_key(id))
}
