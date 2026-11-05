//! Govee adapters for luma: LAN discovery, razer segments and whole-light control.
pub mod catalog;
pub mod lan;
pub mod razer;
pub mod razer_light;
pub mod whole_light;

pub use catalog::{capability, Capability};
