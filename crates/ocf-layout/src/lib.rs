//! ocf-layout — adaptive physical layouts (Phase 7).

#![deny(unsafe_code)]

pub mod candidate;
pub mod layout;
pub mod observer;
pub mod registry;

pub use candidate::generate_candidate;
pub use layout::PhysicalLayout;
pub use observer::WorkloadObserver;
pub use registry::LayoutRegistry;
