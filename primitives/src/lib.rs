//! dWallet Native Blockchain - Primitives
//! 
//! Shared types, constants, and utilities used across all pallets.

#![cfg_attr(not(feature = "std"), no_std)]

// Core modules - always available
pub mod types;
pub mod constants;

// Standard-only modules
#[cfg(feature = "std")]
pub mod crypto;

pub mod math;

// Re-export commonly used items for convenience
pub use types::*;
pub use constants::*;

#[cfg(feature = "std")]
pub use crypto::*;

pub use math::*;
