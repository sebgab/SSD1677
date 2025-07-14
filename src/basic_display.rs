//! This module provides the structures and functionality for managing a display
//! using the SSD1677 controller. It includes definitions for display dimensions,
//! rotation options, and the main [BasicDisplay] struct that interfaces with the hardware.
//!
//! The [BasicDisplay] struct is responsible for initializing the display, resetting it,
//! and updating its contents. It uses a generic interface that implements the
//! [DisplayInterface] and [DisplayCommands] traits, allowing for flexibility in
//! hardware implementations.
use crate::command;
use crate::command::*;
use crate::config::Config;
use crate::interface::DisplayInterface;

/// Maximum number of gate outputs for the display
pub const MAX_GATE_OUTPUTS: u16 = 680;
/// Maximum number of source outputs for the display
pub const MAX_SOURCE_OUTPUTS: u16 = 960;

#[cfg(feature = "defmt")]
#[derive(defmt::Format)]
/// The display's dimensions
pub struct Dimensions {
    /// The number of rows in the display
    /// Must be less than or equal [MAX_GATE_OUTPUTS]
    pub rows: u16,

    /// The number of columns in the display
    /// Must be less than or equal [MAX_SOURCE_OUTPUTS]
    pub cols: u16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg(feature = "defmt")]
#[derive(defmt::Format)]
/// Represents the rotation of the display relative to the native orientation.
pub enum Rotation {
    /// No rotation
    Rotate0,
    /// 90 degrees rotated
    Rotate90,
    /// 180 degrees rotated
    Rotate180,
    /// 270 degrees rotated
    Rotate270,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg(feature = "defmt")]
#[derive(defmt::Format)]
/// The kind of update to do when updating the [BasicDisplay].
///
/// The different enum values take different amount of times, and yield different quality results.
/// - The [Slow] value ensures the entire display is clear and yields a crisp image
/// - The [Fast] value ensures a quick update, but there may be some visual ghosting.
///
///
/// [Slow]: self::DisplayUpdateMode::Slow
/// [Fast]: self::DisplayUpdateMode::Fast
pub enum DisplayUpdateMode {
    /// Perform a "fast" update, this can struggle to clear pixels
    Fast = 0xFF,
    /// Perform a "slow" update, this takes a while, but the result is clean
    Slow = 0xF7,
}

impl Default for Rotation {
    /// Default is no rotation
    fn default() -> Self {
        Rotation::Rotate0
    }
}

/// A configured display with a hardware interface
pub struct BasicDisplay<I, SPI>
where
    SPI: embedded_hal::spi::SpiDevice,
    I: DisplayInterface + DisplayCommands<SPI>,
{
    pub(crate) interface: I,   // The interface for communicating with the display
    pub(crate) config: Config, // The display configuration
    _phantom: core::marker::PhantomData<SPI>, // Phantom data to hold the SPI type
}

impl<I, SPI> BasicDisplay<I, SPI>
where
    I: DisplayInterface + DisplayCommands<SPI>,
    SPI: embedded_hal::spi::SpiDevice,
{
    /// Create a new display instance from a [DisplayInterface] and a [Config].
    ///
    /// The [Config] is created using the [Builder](crate::config::Builder).
    ///
    /// # Arguments
    ///
    /// * `interface` - The interface for communicating with the display.
    /// * `config` - The configuration for the display.
    pub fn new(interface: I, config: Config) -> Self {
        Self {
            interface,
            config,
            // TODO: Figure out if I can remove PhantomData
            _phantom: core::marker::PhantomData,
        }
    }

    /// Reset the display.
    ///
    /// This will perform a hardware reset, followed by a software reset.
    /// This is useful for waking a controller that has entered deep sleep.
    ///
    /// # Arguments
    ///
    /// * `delay` - A delay implementation to use for timing.
    ///
    /// # Returns
    ///
    /// * `Result<(), <I as DisplayInterface>::Error>` - Returns Ok on success, or an error if the reset fails.
    pub fn reset<D: embedded_hal::delay::DelayNs>(
        &mut self,
        delay: &mut D,
    ) -> Result<(), <I as DisplayInterface>::Error> {
        // Perform the hardware reset
        self.interface.reset_hardware(delay);

        // Perform the software reset
        self.interface
            .reset_software()
            .expect("Failed to soft-reset the device");

        // Wait for the display to be ready
        self.interface.busy_wait();

        // Re-initialize the display
        self.init()
    }

    /// Initialize the display controller according to the datasheet.
    ///
    /// This method configures the display settings as specified in the SSD1677 datasheet,
    /// including filling the RAM, setting the driver output control, and loading the waveform LUT.
    ///
    /// # Returns
    ///
    /// * `Result<(), <I as DisplayInterface>::Error>` - Returns Ok on success, or an error if initialization fails.
    pub fn init(&mut self) -> Result<(), <I as DisplayInterface>::Error> {
        // 3. Send intialization code
        // Clear and fill RAM
        self.interface
            .auto_write_ram_black_and_white_regular_pattern(0xF7)
            .expect("Failed to fill BW RAM");
        self.interface
            .auto_write_ram_red_regular_pattern(0xF7)
            .expect("Failed to fill RED RAM");

        // Set gate driver output
        self.interface
            .set_driver_output_control_from_width(self.config.dimensions.rows)
            .expect("Failed to set gate control");

        // Set the data entry mode
        self.interface
            .set_data_entry_mode(
                DataEntryMode::IncrementXIncrementY,
                IncrementAxis::Horizontal,
            )
            .expect("Failed to set data entry mode");

        // Set the display RAM size
        self.interface
            .set_ram_address_based_on_size(self.config.dimensions.rows, self.config.dimensions.cols)
            .expect("Failed to set RAM address");

        // Set the panel border waveform control
        self.interface
            .set_border_waveform_control(
                command::WaveformVDBOption::Transition,
                command::VDBFixedLevelSetting::VSS,
                command::VDBGSTransitionSetting::LUT1,
            )
            .expect("Failed to set waveform control");

        // 4. Load waveform LUT
        // Set temperature sensor
        self.interface
            .set_temperature_sensor(command::TemperatureSensor::Internal)
            .expect("Failed to set temp sensor");
        // Set waveform LUT from OTP
        self.interface
            .update_display_option2(0xFF)
            .expect("Failed to load waveform LUT");
        // Force display refresh
        self.interface
            .refresh_display()
            .expect("Failed to refresh self.interfacelay");

        // Wait for the display to be ready
        self.interface.busy_wait();

        Ok(())
    }

    /// Update the display contents by writing the supplied buffers to the controller.
    ///
    /// This function takes two optional buffers: one for the black and white pixels
    /// and another for the red pixels. If a buffer is provided, it will be written to
    /// the corresponding RAM of the display controller. The function will reset the
    /// RAM address before writing the data and will busy wait until the display refresh
    /// has completed.
    ///
    /// # Arguments
    ///
    /// * `bw_buffer` - an optional slice of bytes representing the black and white pixel data.
    ///                 If `None`, the black and white RAM will not be updated.
    /// * `red_buffer` - An optional slice of bytes representing the red pixel data.
    ///                  If `None`, the red RAM will not be updated.
    /// * `update_mode` - The kond of update to do, see [DisplayUpdateMode]
    ///
    /// # Returns
    ///
    /// * `Result<(), <I as DisplayInterface>::Error>` - Returns `Ok(())` on success, or an error
    ///   if writing to the RAM or refreshing the display fails.
    pub fn update(
        &mut self,
        bw_buffer: Option<&[u8]>,
        red_buffer: Option<&[u8]>,
        update_mode: DisplayUpdateMode,
    ) -> Result<(), <I as DisplayInterface>::Error> {
        // Write the black and white RAM if provided
        if let Some(buffer) = bw_buffer {
            // Reset the address
            self.interface
                .set_ram_x_count(0)
                .expect("Failed to set RAM address for x");
            self.interface
                .set_ram_y_count(0)
                .expect("Failed to set RAM address for y");

            // Copy the data
            self.interface
                .write_ram_black_and_white(buffer)
                .expect("Failed to write black and white RAM buffer");
        }

        // Write the red RAM if provided
        if let Some(buffer) = red_buffer {
            // Reset the address
            self.interface
                .set_ram_x_count(0)
                .expect("Failed to set RAM address for x");
            self.interface
                .set_ram_y_count(0)
                .expect("Failed to set RAM address for y");

            // Copy the data
            self.interface
                .write_ram_red(buffer)
                .expect("Failed to write RED RAM buffer");
        }

        // Set the update mode
        self.interface
            .update_display_option2(update_mode as u8)
            .unwrap();

        // Refresh the display
        self.interface
            .refresh_display()
            .expect("Failed to refresh the display");

        Ok(())
    }

    /// Return the number of rows the display has
    pub fn rows(&self) -> u16 {
        self.config.dimensions.rows
    }

    /// Return the number of cols the display has
    pub fn cols(&self) -> u16 {
        self.config.dimensions.cols
    }

    /// Returns the rotation the display was configured with
    pub fn rotation(&self) -> Rotation {
        self.config.rotation
    }
}
