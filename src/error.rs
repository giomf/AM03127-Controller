extern crate alloc;
use alloc::{
    format,
    string::{String, ToString},
};
use core::fmt::Display;
use esp_storage::FlashStorageError;

/// Error types for the application
///
/// This enum represents all possible errors that can occur in the application.
#[derive(Debug)]
pub enum Error {
    /// Storage-related errors
    Storage(String),
    /// UART communication errors
    Uart(String),
    /// Internal application errors
    Internal(String),
    /// Resource not found errors
    NotFound(String),
    /// Bad request errors (invalid input)
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
        use alloc::format;

        let message = match value {
            sequential_storage::Error::Storage { value, .. } => {
                const PREFIX: &str = "Internal Storage error:";
                match value {
                    FlashStorageError::IoError => format!("{PREFIX} I/O error"),
                    FlashStorageError::IoTimeout => format!("{PREFIX} I/O timeout"),
                    FlashStorageError::CantUnlock => format!("{PREFIX} can not unlock"),
                    FlashStorageError::NotAligned => format!("{PREFIX} not aligned"),
                    FlashStorageError::OutOfBounds => format!("{PREFIX} out of bounds"),
                    FlashStorageError::Other(code) => format!("{PREFIX} {code}"),
                    _ => format!("{PREFIX} unknown error"),
                }
            }
            sequential_storage::Error::FullStorage => "Storage is full".to_string(),
            sequential_storage::Error::Corrupted { .. } => "Storage is corrupted".to_string(),
            sequential_storage::Error::BufferTooBig => {
                "A provided buffer was too big to be used".to_string()
            }
            sequential_storage::Error::BufferTooSmall(needed) => {
                format!("A provided buffer was too small to be used. Needed was {needed}")
            }
            sequential_storage::Error::SerializationError(value) => {
                format!("Map value error: {value}")
            }
            sequential_storage::Error::ItemTooBig => {
                "The item is too big to fit in the flash".to_string()
            }
            _ => "Unknown error".to_string(),
        };

        Self::Storage(message)
    }
}

impl From<esp_hal::uart::IoError> for Error {
    fn from(value: esp_hal::uart::IoError) -> Self {
        match value {
            esp_hal::uart::IoError::Tx(tx_error) => Self::from(tx_error),
            esp_hal::uart::IoError::Rx(rx_error) => Self::from(rx_error),
            _ => Self::Uart("Unknown error".to_string()),
        }
    }
}

impl From<esp_hal::uart::RxError> for Error {
    fn from(value: esp_hal::uart::RxError) -> Self {
        Self::Uart(value.to_string())
    }
}

impl From<esp_hal::uart::TxError> for Error {
    fn from(value: esp_hal::uart::TxError) -> Self {
        Self::Uart(value.to_string())
    }
}
