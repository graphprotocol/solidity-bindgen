// A module for implementation that needs to be exposed to macros
#[doc(hidden)]
pub mod internal;

mod context;

// Re-export the macros
pub use solidity_bindgen_macros::*;

pub use context::Context;
