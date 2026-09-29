//! `captain restart`: a stop, then a start, for example to apply new resources.

use anyhow::Result;

use super::{start, stop};
use crate::context::Context;

pub fn run(context: &Context) -> Result<()> {
    stop::run(context)?;
    start::run(context)
}
