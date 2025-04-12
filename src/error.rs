//! This module defines the error types for the SSD1677 display controller.
//!
//! The [SSD1677Error] enum encapsulates the various errors that can occur
//! when interacting with the SSD1677 display. Currently, it includes:
//!
//! - [SetPinError](self::SSD1677Error::SetPinError): An error that occurs when there is a failure in setting
//!   a pin, which may indicate issues with hardware connections or
//!   configuration.
//!
//! This error handling mechanism allows users of the SSD1677 display driver
//! to gracefully handle and respond to errors that may arise during
//! operation.
pub enum SSD1677Error {
    /// An error that occurs when there is a failure in setting a pin.
    SetPinError,
}

