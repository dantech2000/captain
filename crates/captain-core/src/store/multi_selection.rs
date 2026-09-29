/// How a click changes a [`MultiSelection`], from the keys held during the click.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectMode {
    /// A plain click: select only this row.
    Replace,
    /// Cmd-click on macOS, Ctrl-click elsewhere: add or remove this row.
    Toggle,
    /// Shift-click: select the rows from the anchor to this one.
    Range,
}

impl SelectMode {
    /// The mode for a click with Shift and the platform's secondary key (Cmd on
    /// macOS, Ctrl elsewhere). Shift wins, as in Finder and Explorer.
    pub fn from_keys(shift: bool, secondary: bool) -> Self {
        match (shift, secondary) {
            (true, _) => Self::Range,
            (false, true) => Self::Toggle,
            (false, false) => Self::Replace,
        }
    }
}

/// The rows a list has selected, by key, for bulk actions. The anchor is the last
/// row clicked without Shift; a Shift-click selects from it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MultiSelection {
    keys: Vec<String>,
    anchor: Option<String>,
}

impl MultiSelection {
    /// Applies a click on `key`. `order` is every visible row's key, in list order,
    /// for a range.
    pub fn click(&mut self, key: &str, mode: SelectMode, order: &[String]) {
        let range = match (mode, &self.anchor) {
            (SelectMode::Range, Some(anchor)) => range(order, anchor, key),
            _ => None,
        };
        match (mode, range) {
            (SelectMode::Range, Some(range)) => self.keys = range,
            (SelectMode::Toggle, _) => {
                if self.contains(key) {
                    self.keys.retain(|k| k != key);
                } else {
                    self.keys.push(key.to_string());
                }
                self.anchor = Some(key.to_string());
            }
            _ => {
                self.keys = vec![key.to_string()];
                self.anchor = Some(key.to_string());
            }
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        self.keys.iter().any(|k| k == key)
    }

    /// The selected keys, in the order the user added them.
    pub fn keys(&self) -> &[String] {
        &self.keys
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// True when a bulk action applies: two or more rows.
    pub fn is_bulk(&self) -> bool {
        self.keys.len() > 1
    }

    pub fn clear(&mut self) {
        self.keys.clear();
        self.anchor = None;
    }

    /// Drops the keys `keep` rejects, for example rows that went away.
    pub fn retain(&mut self, keep: impl Fn(&str) -> bool) {
        self.keys.retain(|k| keep(k));
        if self.anchor.as_deref().is_some_and(|a| !keep(a)) {
            self.anchor = None;
        }
    }
}

/// The keys from `from` to `to`, inclusive, in list order. `None` if either is
/// not in `order`.
fn range(order: &[String], from: &str, to: &str) -> Option<Vec<String>> {
    let a = order.iter().position(|k| k == from)?;
    let b = order.iter().position(|k| k == to)?;
    Some(order[a.min(b)..=a.max(b)].to_vec())
}

#[cfg(test)]
mod tests;
