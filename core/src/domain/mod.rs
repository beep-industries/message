use thiserror::Error;

pub mod entities;
pub mod ports;

#[derive(Debug, Error)]
pub enum CoreError {}
