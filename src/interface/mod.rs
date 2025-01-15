//! Synapse Subnet Interface
//! 
//! Provides unified access to subnet functionality through CLI, REST API, and GUI interfaces.
//! Uses SS58 keys for authentication and supports real-time updates via WebSocket.

pub mod api;
pub mod cli;
pub mod gui;
pub mod core;

use self::core::prelude::*;

pub use self::api::ApiServer;
pub use self::cli::Cli;
pub use self::gui::GuiServer;
