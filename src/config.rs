use crate::display::{self, Dimensions, Rotation};

/// Builder for constructing a display config
pub struct Builder {
    dimensions: Option<Dimensions>,
    // rotation: Rotation,
}

/// Display configuration
///
/// Passed to `Display::new`. Use `Builder` to construct a `Config`.
pub struct Config {
    pub(crate) dimensions: Dimensions,
    // pub(crate) rotation: Rotation,
}

/// Error returned by invalid Builder configuration
///
/// Only returned if configuration is built without dimensions.
#[derive(Debug)]
pub struct BuilderError {}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            dimensions: None,
            // rotation: Rotation::default(),
        }
    }
}

impl Builder {
    /// Create a new `Builder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the display dimensions
    ///
    /// There is no default for this setting. The dimensions must be set for the builder to
    /// successfully build a `Config`.
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

    /// Set the display rotation
    ///
    /// Defaults to no ratation
    // pub fn rotation(self, rotation: Rotation) -> Self {
    //     Self { rotation, ..self }
    // }

    /// Build the display econfig
    ///
    /// Wll fail if dimensions are not set.
    pub fn build(self) -> Result<Config, BuilderError> {
        Ok(Config {
            dimensions: self.dimensions.ok_or_else(|| BuilderError {})?,
            // rotation: self.rotation,
        })
    }
}
