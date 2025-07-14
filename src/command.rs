//! This module defines the commands to the [BasicDisplay](crate::basic_display::BasicDisplay) and the valid options to those commands.
use crate::interface::{DisplayInterface, Interface4Pin};

/// The address increment orientation when writing image data.
/// This configures how the controller auto-increments the row and column address when data is
/// written using the WriteImageData command.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum IncrementAxis {
    /// X direction
    Horizontal = 0b0,
    /// Y direction
    Vertical = 0b1,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum DataEntryMode {
    DecrementXDecrementY = 0b00,
    IncrementXDecrementY = 0b01,
    DecrementXIncrementY = 0b10,
    IncrementXIncrementY = 0b11,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum TemperatureSensor {
    Internal = 0x80,
    External = 48,
}

/// Ram display update option, see page 27 in the datasheet
#[derive(Clone, Copy)]
pub enum RamOption {
    Normal = 0b0000,
    Bypass = 0b0100,
    Invert = 0b1000,
}

#[derive(Clone, Copy)]
pub enum DeepSleepMode {
    /// Not sleeping
    Normal,
    /// Deep sleep with RAM preserved
    PreserveRAM,
    /// Deep sleep with RAM discarded
    DiscardRAM,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum BoosterInrush {
    Level1 = 0x40,
    Level2 = 0x80,
}

/// Select VBD option
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum WaveformVDBOption {
    /// Use the GS transision defined by `VDBGSTransitionSetting`
    Transition = 0b00,
    /// Use the fixed level defined by `VDBFixedLevelSetting`
    Fixed = 0b01,
    VCOM = 0b10,
    /// POR Value
    HiZ = 0b11,
}

/// Fix Level Setting for VBD
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum VDBFixedLevelSetting {
    /// POR
    VSS = 0b00,
    VSH1 = 0b01,
    VSL = 0b10,
    VSH2 = 0b11,
}

/// GS Transition setting for VBD
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum VDBGSTransitionSetting {
    /// POR value
    LUT0 = 0b00,
    LUT1 = 0b01,
    LUT2 = 0b10,
    LUT3 = 0b11,
}

/// The commands implemented on the display
pub trait DisplayCommands<SPI>
where
    SPI: embedded_hal::spi::SpiDevice,
{
    fn set_driver_output_control(
        &mut self,
        max_gate_lines: u16,
        scanning_sequence_and_direction: u8,
    ) -> Result<(), SPI::Error>;

    fn set_driver_output_control_from_width(&mut self, width: u16) -> Result<(), SPI::Error>;

    fn set_data_entry_mode(
        &mut self,
        data_entry_mode: DataEntryMode,
        increment_axis: IncrementAxis,
    ) -> Result<(), SPI::Error>;

    fn write_ram_black_and_white(&mut self, data: &[u8]) -> Result<(), SPI::Error>;

    fn write_ram_red(&mut self, data: &[u8]) -> Result<(), SPI::Error>;

    fn auto_write_ram_red_regular_pattern(&mut self, value: u8) -> Result<(), SPI::Error>;

    fn auto_write_ram_black_and_white_regular_pattern(
        &mut self,
        value: u8,
    ) -> Result<(), SPI::Error>;

    fn set_ram_x_count(&mut self, offset: u16) -> Result<(), SPI::Error>;

    fn set_ram_y_count(&mut self, offset: u16) -> Result<(), SPI::Error>;

    fn refresh_display(&mut self) -> Result<(), SPI::Error>;

    fn set_ram_x_address(&mut self, start: u16, end: u16) -> Result<(), SPI::Error>;

    fn set_ram_y_address(&mut self, start: u16, end: u16) -> Result<(), SPI::Error>;

    fn set_ram_address_based_on_size(&mut self, width: u16, height: u16) -> Result<(), SPI::Error>;

    fn nop(&mut self) -> Result<(), SPI::Error>;

    fn set_gate_driving_voltage(&mut self, voltage: f32) -> Result<(), SPI::Error>;

    fn set_source_driving_voltage(
        &mut self,
        vsh1_voltage: f32,
        vsh2_voltage: f32,
        vsl_voltage: f32,
    ) -> Result<(), SPI::Error>;

    fn update_display_option1(
        &mut self,
        black_and_white_option: RamOption,
        red_option: RamOption,
    ) -> Result<(), SPI::Error>;

    fn update_display_option2(&mut self, option: u8) -> Result<(), SPI::Error>;

    fn reset_hardware<D: embedded_hal::delay::DelayNs>(&mut self, delay: &mut D);

    fn reset_software(&mut self) -> Result<(), SPI::Error>;

    fn set_border_waveform_control(
        &mut self,
        vdb_option: WaveformVDBOption,
        fixed_level_setting: VDBFixedLevelSetting,
        transition_setting: VDBGSTransitionSetting,
    ) -> Result<(), SPI::Error>;

    fn set_temperature_sensor(&mut self, sensor: TemperatureSensor) -> Result<(), SPI::Error>;

    fn set_booster_soft_start_control(&mut self, inrush: BoosterInrush) -> Result<(), SPI::Error>;
}

/// A command that can be issued to the SSD1677 controller
impl<SPI, OUT, IN> DisplayCommands<SPI> for Interface4Pin<SPI, OUT, IN>
where
    SPI: embedded_hal::spi::SpiDevice,
    OUT: embedded_hal::digital::OutputPin,
    IN: embedded_hal::digital::InputPin,
{
    /// Set the MUX of gate lines, scanning sequence and direction
    fn set_driver_output_control(
        &mut self,
        max_gate_lines: u16,
        scanning_sequence_and_direction: u8,
    ) -> Result<(), SPI::Error> {
        self.send_command(0x01)?;
        let [upper, lower] = max_gate_lines.to_le_bytes();
        self.send_data(&[upper, lower, scanning_sequence_and_direction])?;

        Ok(())
    }

    fn set_driver_output_control_from_width(&mut self, width: u16) -> Result<(), SPI::Error> {
        // This command set is based on the example code for the STM32 from here:
        // https://www.good-display.com/product/457.html
        self.send_command(0x01)?;
        self.send_data(&[((width - 1) % 256).try_into().unwrap()])?;
        self.send_data(&[((width - 1) / 256).try_into().unwrap()])?;
        self.send_data(&[0x02])?;

        Ok(())
    }

    /// Define the data entry mode settings
    fn set_data_entry_mode(
        &mut self,
        data_entry_mode: DataEntryMode,
        increment_axis: IncrementAxis,
    ) -> Result<(), SPI::Error> {
        // Send the config command
        self.send_command(0x11)?;

        // Structure the config data
        let config_option: u8 = ((increment_axis as u8) << 2) | data_entry_mode as u8;

        // Send the config
        self.send_data(&[config_option])?;

        Ok(())
    }

    /// Write data to the black and white RAM buffer
    fn write_ram_black_and_white(&mut self, data: &[u8]) -> Result<(), SPI::Error> {
        self.send_command(0x24)?;
        self.send_data(data)?;
        Ok(())
    }

    /// Write data to the red RAM buffer
    fn write_ram_red(&mut self, data: &[u8]) -> Result<(), SPI::Error> {
        self.send_command(0x26)?;
        self.send_data(data)?;
        Ok(())
    }

    /// Fill the red RAM buffer with a single value
    fn auto_write_ram_red_regular_pattern(&mut self, value: u8) -> Result<(), SPI::Error> {
        self.send_command(0x46)?;
        self.send_data(&[value])?;
        Ok(())
    }

    /// Fill the black and white RAM buffer with a single value
    fn auto_write_ram_black_and_white_regular_pattern(
        &mut self,
        value: u8,
    ) -> Result<(), SPI::Error> {
        self.send_command(0x47)?;
        self.send_data(&[value])?;
        Ok(())
    }

    /// Set the current X axis count
    fn set_ram_x_count(&mut self, offset: u16) -> Result<(), SPI::Error> {
        self.send_command(0x4E)?;
        self.send_data(&offset.to_le_bytes())?;
        Ok(())
    }

    /// Set the current Y axis count
    fn set_ram_y_count(&mut self, offset: u16) -> Result<(), SPI::Error> {
        self.send_command(0x4F)?;
        self.send_data(&offset.to_le_bytes())?;
        Ok(())
    }

    fn refresh_display(&mut self) -> Result<(), SPI::Error> {
        // Send the refesh command
        self.send_command(0x20)?;
        self.busy_wait();
        Ok(())
    }

    /// Specify the start/end positions of the window address in the X direction by an address unit
    /// for RAM.
    ///
    /// # Note
    /// Start any end values are 10-bit, bit ranges 11-16 will be discarded.
    fn set_ram_x_address(&mut self, start: u16, end: u16) -> Result<(), SPI::Error> {
        // Split the input value to bytes
        let [start_hi, start_lo] = start.to_le_bytes();
        let [end_hi, end_lo] = end.to_le_bytes();

        // Create the data
        let data = [start_hi, start_lo, end_hi, (end_lo & 0b00111111)];

        self.send_command(0x44)?;
        self.send_data(&data)?;

        Ok(())
    }

    /// Specify the start/end positions of the window address in the Y direction by an address unit
    /// for RAM.
    ///
    /// # Note
    /// Start any end values are 10-bit, bit ranges 11-16 will be discarded.
    fn set_ram_y_address(&mut self, start: u16, end: u16) -> Result<(), SPI::Error> {
        // Split the input value to bytes
        let [start_hi, start_lo] = start.to_le_bytes();
        let [end_hi, end_lo] = end.to_le_bytes();

        // Create the data
        let data = [start_hi, start_lo, end_hi, (end_lo & 0b00111111)];

        self.send_command(0x45)?;
        self.send_data(&data)?;

        Ok(())
    }

    /// Set the start and end RAM addresses for both X and Y based on the display dimentions given
    fn set_ram_address_based_on_size(&mut self, width: u16, height: u16) -> Result<(), SPI::Error> {
        self.set_ram_x_address(0, height - 1)?;
        self.set_ram_y_address(0, width - 1)?;

        Ok(())
    }

    // No operation instruction, does nothing.
    // It can be used to terminate Frame Memory Write or Read commands
    fn nop(&mut self) -> Result<(), SPI::Error> {
        self.send_command(0x7F)?;

        Ok(())
    }

    /// Set the gate driving voltage
    /// Valid values are between 12 and 20 in increments of 0.5 volts
    fn set_gate_driving_voltage(&mut self, voltage: f32) -> Result<(), SPI::Error> {
        // Validate that it is within range
        // If not, set the voltage to the POR value of 20V
        let value: u8 = match voltage {
            12.0 => 0x07,
            12.5 => 0x08,
            13.0 => 0x09,
            13.5 => 0x0A,
            14.0 => 0x0B,
            14.5 => 0x0C,
            15.0 => 0x0D,
            15.5 => 0x0E,
            16.0 => 0x0F,
            16.5 => 0x10,
            17.0 => 0x11,
            17.5 => 0x12,
            18.0 => 0x13,
            18.5 => 0x14,
            19.0 => 0x15,
            19.5 => 0x16,
            20.0 => 0x17,
            _ => 0, // POR value, also 20V
        };

        self.send_command(0x03)?;
        self.send_data(&[value])?;

        Ok(())
    }

    /// Set the source driving voltage
    ///
    /// # NOTE
    /// VSH1 must be larger than VSH2
    /// Valid voltage range:
    ///     VSH1 = 9 to 17 (increments of 0.2)
    ///     VSH2 = 2.4 to 17 (increments of 0.1 between 2.4 and 9 V, increments of 0.2 from
    ///     thereon)
    ///     VSL = -9 to -17 (increments of 0.5)
    fn set_source_driving_voltage(
        &mut self,
        vsh1_voltage: f32,
        vsh2_voltage: f32,
        vsl_voltage: f32,
    ) -> Result<(), SPI::Error> {
        todo!();
    }

    /// Set RAM content options for update display command.
    fn update_display_option1(
        &mut self,
        black_and_white_option: RamOption,
        red_option: RamOption,
    ) -> Result<(), SPI::Error> {
        // Create the data value
        let data: u8 = (red_option as u8 & 0b1111) << 4     //Set the red option
        | (black_and_white_option as u8 & 0b1111); // Set the BW opiton

        // Send the command and data
        self.send_command(0x21)?;
        self.send_data(&[data])?;

        Ok(())
    }

    /// Set display update sequence option
    /// See datasheet entry for what values mean
    fn update_display_option2(&mut self, option: u8) -> Result<(), SPI::Error> {
        self.send_command(0x22)?;
        self.send_data(&[option])?;

        Ok(())
    }

    /// Perform a hardware reset
    fn reset_hardware<D: embedded_hal::delay::DelayNs>(&mut self, delay: &mut D) {
        use crate::interface::RESET_DELAY_MS;

        // Disable the display, the wait for the controller to catch up
        self.reset_pin.set_low().unwrap();
        delay.delay_ms(RESET_DELAY_MS.into());
        // Enable the display, the wait for the controller to catch up
        self.reset_pin.set_high().unwrap();
        delay.delay_ms(RESET_DELAY_MS.into());
    }

    /// Perform a software reset.
    /// This resets all parameters except deep sleep mode to their default values.
    /// RAM content is not affected.
    /// BUSY will be high while reset is in progress
    fn reset_software(&mut self) -> Result<(), SPI::Error> {
        // Tell the device to soft reset
        self.send_command(0x12)?;

        // Wait for the soft reset to be over
        self.busy_wait();

        Ok(())
    }

    /// Select border waveform for VBD
    fn set_border_waveform_control(
        &mut self,
        vdb_option: WaveformVDBOption,
        fixed_level_setting: VDBFixedLevelSetting,
        transition_setting: VDBGSTransitionSetting,
    ) -> Result<(), SPI::Error> {
        self.send_command(0x3C)?;

        // Create the data packet
        let data = ((vdb_option as u8) << 6)
            | ((fixed_level_setting as u8) << 4)
            | (transition_setting as u8);

        self.send_data(&[data])?;

        Ok(())
    }

    /// Specify which temperature sensor the display uses
    fn set_temperature_sensor(&mut self, sensor: TemperatureSensor) -> Result<(), SPI::Error> {
        self.send_command(0x18)?;
        self.send_data(&[sensor as u8])?;
        Ok(())
    }

    /// Control the inrush current for the booster
    fn set_booster_soft_start_control(&mut self, inrush: BoosterInrush) -> Result<(), SPI::Error> {
        // Frist four bytes are always the same as per datasheet page 24
        // Last bytes depend on inrush mode, these are defined in the enum
        let control_value: [u8; 5] = [0xAE, 0xC7, 0xC3, 0xC0, inrush as u8];

        self.send_command(0x0C)?;
        self.send_data(&control_value)?;

        Ok(())
    }

    /*




    /// Set the deep sleep mode
    DeepSleepMode(DeepSleepMode),

    /// Set the data entry mode and the increment axis
    DataEntryMode(DataEntryMode, IncrementAxis),



    /// Write to temperature sensor register
    /// Please note that the register is 12-bit
    WriteTemperatureSensorControl(u16),

    /// Read temperature sensor control register
    ReadTemperatureSensorControl,

    /// Activate the display update sequence.
    /// BUSY will be high while this is in progress.
    UpdateDisplay,

    */
}
