use thiserror::Error;

pub mod entities;
pub mod ports;
pub mod services;

#[derive(Debug, Error)]
pub enum CoreError {}
