/// A module for implementation that needs to be exposed to macros
#[doc(hidden)]
pub mod internal;

mod context;
mod providers;
mod secrets;
mod web3_provider;

pub use secrets::SafeSecretKey;

pub use providers::{CallProvider, SendProvider};
pub use web3_provider::Web3Provider;

// Re-export the macros
pub use solidity_bindgen_macros::*;

pub use context::{Context, Web3Context};
