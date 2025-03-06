use crate::interface::{DisplayInterface, Interface4Pin};

// The amount of gates the SSD1677 controls, see chapter 1 in the datasheet
const MAX_GATES: u16 = 680;

trait Contains<C>
where
    C: Copy + PartialOrd,
{
    fn contains(&self, item: C) -> bool;
}

/// The address increment orientation when writing image data.
/// This configures ohw the c4)roller auto-increments the row and column address when data is
/// written using the `WriteImageData` command.
#[derive(Clone, Copy)]
pub enum IncrementAxis {
    /// X direction
    Horizontal,
    /// Y direction
    Vertical,
}

#[derive(Clone, Copy)]
pub enum DataEntryMode {
    DecrementXDecrementY,
    IncrementXDecrementY,
    DecrementXIncrementY,
    IncrementXIncrementY,
}

#[derive(Clone, Copy)]
pub enum TemperatureSensor {
    Internal,
    External,
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
pub enum BoosterInrush {
    Level1,
    Level2,
}

/// A command that can be issued to the SSD1677 controller
impl<SPI, OUT, IN> Interface4Pin<SPI, OUT, IN>
where
    SPI: embedded_hal::spi::SpiDevice,
    OUT: embedded_hal::digital::OutputPin,
    IN: embedded_hal::digital::InputPin,
{
    /// Set the MUX of gate lines, scanning sequence and direction
    pub fn set_driver_output_control(
        &mut self,
        max_gate_lines: u16,
        scanning_sequence_and_direction: u8,
    ) -> Result<(), SPI::Error> {
        self.send_command(0x01)?;
        let [upper, lower] = max_gate_lines.to_be_bytes();
        self.send_data(&[upper, lower, scanning_sequence_and_direction])?;

        Ok(())
    }

    pub fn write_ram_black_and_white(&mut self, data: &[u8]) -> Result<(), SPI::Error> {
        self.send_command(0x24)?;
        self.send_data(data)?;
        Ok(())
    }

    pub fn refresh_display(&mut self) -> Result<(), SPI::Error> {
        // Send the refesh command
        self.send_command(0x20)?;
        self.busy_wait();
        Ok(())
    }

    // Specify the start/end positions of the window address in the X direction by an address unit
    // for RAM.
    //
    // # Note
    // Start any end values are 10-bit, bit ranges 11-16 will be discarded.
    pub fn set_ram_x_address(&mut self, start: u16, end: u16) -> Result<(), SPI::Error> {
        // Split the input value to bytes
        let [start_hi, start_lo] = start.to_be_bytes();
        let [end_hi, end_lo] = end.to_be_bytes();

        // Create the data
        let data = [start_lo, start_hi, (end_lo & 0b00111111), end_hi];

        self.send_command(0x44)?;
        self.send_data(&data)?;

        Ok(())
    }

    // Specify the start/end positions of the window address in the Y direction by an address unit
    // for RAM.
    //
    // # Note
    // Start any end values are 10-bit, bit ranges 11-16 will be discarded.
    pub fn set_ram_y_address(&mut self, start: u16, end: u16) -> Result<(), SPI::Error> {
        // Split the input value to bytes
        let [start_hi, start_lo] = start.to_be_bytes();
        let [end_hi, end_lo] = end.to_be_bytes();

        // Create the data
        let data = [start_lo, start_hi, (end_lo & 0b00111111), end_hi];

        self.send_command(0x45)?;
        self.send_data(&data)?;

        Ok(())
    }

    // No operation instruction, does nothing.
    // It can be used to terminate Frame Memory Write or Read commands
    pub fn nop(&mut self) -> Result<(), SPI::Error> {
        self.send_command(0x7F)?;

        Ok(())
    }

    /// Set the gate driving voltage
    /// Valid values are between 12 and 20 in increments of 0.5 volts
    pub fn set_gate_driving_voltage(&mut self, voltage: f32) -> Result<(), SPI::Error> {
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
    pub fn set_source_driving_voltage(
        &mut self,
        vsh1_voltage: f32,
        vsh2_voltage: f32,
        vsl_voltage: f32,
    ) -> Result<(), SPI::Error> {
        todo!();
    }

    /// Set RAM content options for update display command.
    pub fn update_display_option1(
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
    pub fn update_display_option2(&mut self, option: u8) -> Result<(), SPI::Error> {
        self.send_command(0x22)?;
        self.send_data(&[option])?;

        Ok(())
    }

    /*



    /// Control the inrush current for the booster
    BoosterSoftStartControl(BoosterInrush),

    /// Set the deep sleep mode
    DeepSleepMode(DeepSleepMode),

    /// Set the data entry mode and the increment axis
    DataEntryMode(DataEntryMode, IncrementAxis),

    /// Perform a software reset.
    /// This resets all parameters except deep sleep mode to their default values.
    /// RAM content is not affected.
    /// BUSY will be high while reset is in progress
    SoftReset,

    /// Specify which temperature sensor the display uses
    TemperatureSensorSelection(TemperatureSensor),

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
