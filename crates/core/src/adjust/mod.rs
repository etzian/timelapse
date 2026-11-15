//! Phase 8 (ADR-025): deflicker and stabilization. Per-frame values are computed for a
//! saved selection, stored in SQLite, and applied by the export while it prepares frames.

pub mod deflicker;
pub mod stabilize;
