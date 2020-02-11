#![feature(fixed_size_array)]

//! This crate provides high-level API to control Dobot robot arms.

pub mod base;
pub mod error;
pub mod message;

pub use base::{Dobot, Mode, Pose};
