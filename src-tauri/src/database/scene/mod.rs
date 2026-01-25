//! Scene database operations.
//!
//! This module is split into submodules for better organization:
//! - `crud`: Basic create, read, update, delete operations
//! - `history`: Scene version history management
//! - `operations`: Split, merge, and duplicate operations
//! - `mapping`: Row mapping helpers

mod crud;
mod history;
mod mapping;
mod operations;
