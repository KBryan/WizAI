//! MCP (Model Context Protocol) integration for Spree
//!
//! This module enables Spree to connect to MCP servers like Repliers

pub mod client;
pub mod protocol;
pub mod servers;

pub use client::*;
pub use protocol::*;
pub use servers::*;
