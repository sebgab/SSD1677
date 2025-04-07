use crate::command;
use crate::command::*;
use crate::config::Config;
use crate::interface::{self, DisplayInterface};

pub const MAX_GATE_OUTPUTS: u16 = 680;
pub const MAX_SOURCE_OUTPUTS: u16 = 960;

/// The display's dimensions
pub struct Dimensions {
    /// The number of rows in the display
    /// Must be less than or equal MAX_GATE_OUTPUTS
    pub rows: u16,

    /// The number of columns in the display
    /// Must be less than or equal MAX_SOURCE_OUTPUTS
    pub cols: u16,
}

/// Represents the rotation of the display relative to the native orientation.
pub enum Rotation {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

impl Default for Rotation {
    /// Default is no rotation
    fn default() -> Self {
        Rotation::Rotate0
    }
}

/// A configured display with a hardware interface
pub struct Display<I, SPI>
where
    SPI: embedded_hal::spi::SpiDevice,
    I: DisplayInterface + DisplayCommands<SPI>,
{
    interface: I,
    config: Config,
    _phantom: core::marker::PhantomData<SPI>,
}

impl<I, SPI> Display<I, SPI>
where
    I: DisplayInterface + DisplayCommands<SPI>,
    SPI: embedded_hal::spi::SpiDevice,
{
    /// Create a new display instance from a `DisplayInterface` and a `Config`.
    ///
    /// The `Config` is created with `config::Builder`.
    pub fn new(interface: I, config: Config) -> Self {
        Self {
            interface,
            config,
            // TODO: Figure out if I can remove PhantomData
            _phantom: core::marker::PhantomData,
        }
    }

    /// Reset the display.
    /// This will perform a hardware reset, followed by a software reset.
    ///
    /// This will wake a controller that has entered deep sleep.
    pub fn reset<D: embedded_hal::delay::DelayNs>(
        &mut self,
        delay: &mut D,
    ) -> Result<(), <I as DisplayInterface>::Error> {
        // Do the hardware reset
        self.interface.reset_hardware(delay);

        // Do the software reset
        self.interface
            .reset_software()
            .expect("Failed to soft-reset the device");

        // Wait for the display to be ready
        self.interface.busy_wait();

        // Re-initialize the display
        self.init()
    }

    /// Initialize the display controller according to the datasheet
    /// See SSD1677 datasheet chapter 9
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
            .set_driver_output_control_from_width(self.config.dimensions.cols)
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
            .set_ram_address_based_on_size(self.config.dimensions.cols, self.config.dimensions.rows)
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

    /// Update the display contents by writing the supplied buffers to the controller
    ///
    /// This will write the two buffers provided to the controller RAM and initializet he update.
    /// The function will busy wait until the refresh has completed.
    pub fn update(
        &mut self,
        bw_buffer: Option<&[u8]>,
        red_buffer: Option<&[u8]>,
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

        // Set the "slow" update mode
        self.interface.update_display_option2(0xF7).unwrap();

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

    // /// Returns the rotation the display was configured with
    // pub fn rotation(&self) -> Rotation {
    //
    // }
}
