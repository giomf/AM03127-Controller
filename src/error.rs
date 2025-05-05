use core::fmt::{Display, Write};
use esp_storage::FlashStorageError;

use crate::String;

#[derive(Debug)]
pub enum Error {
    Storage(String),
    Uart(String),
    Internal(String),
    NotFound(String),
    BadRequest(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Storage(message) => write!(f, "Storage: {message}"),
            Error::Uart(message) => write!(f, "Uart: {message}"),
            Error::Internal(message) => write!(f, "Internal: {message}"),
            Error::NotFound(message) => write!(f, "Not found: {message}"),
            Error::BadRequest(message) => write!(f, "Bad request: {message}"),
        }
    }
}

impl core::error::Error for Error {}

impl From<sequential_storage::Error<FlashStorageError>> for Error {
    fn from(value: sequential_storage::Error<FlashStorageError>) -> Self {
        let mut message = String::new();
        match value {
            sequential_storage::Error::Storage { value, .. } => {
                const PREFIX: &str = "Internal Storage error:";
                match value {
                    FlashStorageError::IoError => write!(message, "{PREFIX} I/O error"),
                    FlashStorageError::IoTimeout => write!(message, "{PREFIX} I/O timeout"),
                    FlashStorageError::CantUnlock => write!(message, "{PREFIX} can not unlock"),
                    FlashStorageError::NotAligned => write!(message, "{PREFIX} not aligned"),
                    FlashStorageError::OutOfBounds => write!(message, "{PREFIX} out of bounds"),
                    FlashStorageError::Other(code) => write!(message, "{PREFIX} {code}"),
                    _ => write!(message, "{PREFIX} unknown error"),
                }
            }
            sequential_storage::Error::FullStorage => write!(message, "Storage is full"),
            sequential_storage::Error::Corrupted { .. } => write!(message, "Storage is corrupted"),
            sequential_storage::Error::BufferTooBig => {
                write!(message, "A provided buffer was to big to be used")
            }
            sequential_storage::Error::BufferTooSmall(needed) => write!(
                message,
                "A provided buffer was to small to be used. Needed was {needed}"
            ),
            sequential_storage::Error::SerializationError(value) => {
                write!(message, "Map value error: {value}")
            }
            sequential_storage::Error::ItemTooBig => {
                write!(message, "The item is too big to fit in the flash")
            }
            _ => write!(message, "Unknown error"),
        }
        .expect("Failed to write error message");
        Self::Storage(message)
    }
}

impl From<esp_hal::uart::IoError> for Error {
    fn from(value: esp_hal::uart::IoError) -> Self {
        match value {
            esp_hal::uart::IoError::Tx(tx_error) => Self::from(tx_error),
            esp_hal::uart::IoError::Rx(rx_error) => Self::from(rx_error),
            _ => Self::Uart("Unknown error".into()),
        }
    }
}

impl From<esp_hal::uart::RxError> for Error {
    fn from(value: esp_hal::uart::RxError) -> Self {
        let mut message = String::new();
        write!(message, "{value}").expect("Failed to write message");
        Self::Uart(message)
    }
}

impl From<esp_hal::uart::TxError> for Error {
    fn from(value: esp_hal::uart::TxError) -> Self {
        let mut message = String::new();
        write!(message, "{value}").expect("Failed to write message");
        Self::Uart(message)
    }
}
