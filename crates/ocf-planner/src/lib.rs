//! ocf-planner — dataset profiling and compression plan selection (Phase 5).

#![deny(unsafe_code)]

pub mod candidate;
pub mod cost;
pub mod plan;
pub mod planner;
pub mod profile;
pub mod store;

pub use cost::{CostModel, HardConstraints};
pub use plan::CompressionPlan;
pub use planner::Planner;
pub use profile::DatasetProfile;
pub use store::{load_plan, save_plan};
