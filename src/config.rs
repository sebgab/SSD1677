//! Configuration options when using `Display`.
//!
//! This module provides a builder pattern for constructing a display configuration
//! that can be passed to the `Display::new` function. The `Builder` struct allows
//! users to specify the dimensions and rotation of the display, ensuring that all
//! necessary parameters are set before creating a `Config`.
use crate::display::{self, Dimensions, Rotation};

/// Builder for constructing a display config
pub struct Builder {
    dimensions: Option<Dimensions>,
    rotation: Rotation,
}

/// Display configuration.
///
/// This struct holds the configuration options for the display, including its dimensions
/// and rotation. It is created using the [Builder] and passed to the
/// [`Display::new`](crate::display::Display::new()) function
/// to initialize a new display instance.
pub struct Config {
    pub(crate) dimensions: Dimensions,
    pub(crate) rotation: Rotation,
}

/// Error returned by invalid Builder configuration.
///
/// This error is returned if the configuration is built without specifying the dimensions.
#[derive(Debug)]
pub struct BuilderError {}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            dimensions: None,
            rotation: Rotation::default(),
        }
    }
}

impl Builder {
    /// Create a new `Builder`.
    ///
    /// This initializes a new `Builder` instance with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the display dimensions.
    ///
    /// This method allows the user to specify the dimensions of the display. It is important
    /// to note that there is no default for this setting; the dimensions must be set for the
    /// builder to successfully build a `Config`.
    ///
    /// # Panics
    ///
    /// This method will panic if the specified dimensions do not meet the following criteria:
    /// - The number of columns must be evenly divisible by 8.
    /// - The number of rows must be less than or equal to `display::MAX_GATE_OUTPUTS`.
    /// - The number of columns must be less than or equal to `display::MAX_SOURCE_OUTPUTS`.
    ///
    /// # Arguments
    ///
    /// * `dimensions` - The dimensions of the display to be set.
    pub fn dimensions(self, dimensions: Dimensions) -> Self {
        // Validate that we have valid dimensions
        assert!(
            dimensions.cols % 8 == 0,
            "Columns must be evenly divisibly by 8"
        ); // TODO: Figure out if this is required for SSD1677, or if it is just for SSD1675

        assert!(
            dimensions.rows <= display::MAX_GATE_OUTPUTS,
            "rows must be less thn MAX_GATE_OUTPUTS"
        );

        assert!(
            dimensions.cols <= display::MAX_SOURCE_OUTPUTS,
            "cols must be less than MAX_SOURCE_OUTPUTS"
        );

        Self {
            dimensions: Some(dimensions),
            ..self
        }
    }

    /// Set the display rotation.
    ///
    /// This method allows the user to specify the rotation of the display. The default
    /// rotation is no rotation.
    ///
    /// # Arguments
    ///
    /// * `rotation` - The rotation setting for the display.
    pub fn rotation(self, rotation: Rotation) -> Self {
        Self { rotation, ..self }
    }

    /// Build the display configuration.
    ///
    /// This method constructs a `Config` instance from the builder. It will fail if the
    /// dimensions have not been set, returning a `BuilderError`.
    ///
    /// # Returns
    ///
    /// * `Result<Config, BuilderError>` - A result containing the built configuration or an error.
    pub fn build(self) -> Result<Config, BuilderError> {
        Ok(Config {
            dimensions: self.dimensions.ok_or_else(|| BuilderError {})?,
            rotation: self.rotation,
        })
    }
}
