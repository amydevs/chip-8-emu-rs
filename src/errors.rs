use cpal::BuildStreamError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BeeperError {
    #[error("No default output device found")]
    NoDefaultOutputDevice,
    #[error("Error occured whilst building stream: {0}")]
    BuildStream(#[from] BuildStreamError),
}