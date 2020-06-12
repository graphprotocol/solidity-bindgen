/// A module for implementation that needs to be exposed to macros
#[doc(hidden)]
pub mod internal;

mod context;
mod secrets;

pub use secrets::SafeSecretKey;

// Re-export the macros
pub use solidity_bindgen_macros::*;

pub use context::Context;
