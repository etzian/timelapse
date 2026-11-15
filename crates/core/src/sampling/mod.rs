//! Phase 4: temporal sampling (PROCESSING_PIPELINE.md §6, PRD §5.6, ADR-010).
//!
//! [`select`] holds the pure, deterministic algorithms; [`store`] applies the input rules
//! (every photograph except rejected and quarantined ones; flagged ones optional; no
//! capture time → excluded and counted) and persists runs. A run is an ordered list in
//! SQLite; no file is written.

pub mod select;
pub mod store;

use std::path::Path;

use crate::Result;
use crate::import::open_root;

pub use select::{Method, Policy};
pub use store::{
    Config, DayCount, Excluded, RunPage, RunSummary, SampledItem, SamplingPreview, VerifyReport,
};

pub fn preview_for_root(root: &Path, cfg: &Config) -> Result<SamplingPreview> {
    store::preview(&open_root(root)?, cfg)
}

pub fn create_run_for_root(root: &Path, cfg: &Config) -> Result<RunSummary> {
    store::create_run(&open_root(root)?, cfg)
}

pub fn runs_for_root(root: &Path) -> Result<Vec<RunSummary>> {
    store::list_runs(&open_root(root)?)
}

pub fn run_items_for_root(root: &Path, run: i64) -> Result<Vec<SampledItem>> {
    store::run_items(&open_root(root)?, run)
}

pub fn run_page_for_root(root: &Path, run: i64, offset: usize, limit: usize) -> Result<RunPage> {
    store::run_page(&open_root(root)?, run, offset, limit)
}

pub fn verify_run_for_root(root: &Path, run: i64) -> Result<VerifyReport> {
    store::verify_run(&open_root(root)?, run)
}

#[cfg(test)]
mod exit_tests;
