#[cfg(feature = "tui")]
use anyhow::Result;

#[cfg(feature = "tui")]
use crate::model::{CaseResult, Summary};

#[cfg(feature = "tui")]
pub fn launch(_seed: u64, _runs: u32, _results: &[CaseResult], _summary: &Summary) -> Result<()> {
    // Full ratatui cockpit code goes here.
    Ok(())
}
