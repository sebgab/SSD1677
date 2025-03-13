//! Hardware interface to the display
//!
//! This file implements the functions required to communicate and interface with the SSD1677

use embedded_hal;

// 10ms reset delay as seen in box 2 in chapter 9.1 in the SSD1677 datasheet
pub const RESET_DELAY_MS: u8 = 10;

/// Trait implemented by displays for core functionality
pub trait DisplayInterface {
    type Error;

    /// Send a command to the display controller
    fn send_command(&mut self, command: u8) -> Result<(), Self::Error>;

    /// Send data for a command
    fn send_data(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Reset the controller
    fn reset<D: embedded_hal::delay::DelayNs>(&mut self, delay: &mut D);

    /// Wait for the controller to indicate that it is not busy
    fn busy_wait(&mut self);
}

/// Interface to the SSD1677 driver operating in 4pin SPI mode
pub struct Interface4Pin<SPI, OUT, IN> {
    /// The SpiDevice to communicate with the display
    spi: SPI,
    /// Data / Command pin, 0=command, 1=data
    data_command_pin: OUT,
    /// The reset pin for the display
    pub reset_pin: OUT,
    /// The pin from the controller indicating busy
    busy_pin: IN,
}

// Implement the interface functions
impl<SPI, OUT, IN> Interface4Pin<SPI, OUT, IN>
where
    SPI: embedded_hal::spi::SpiDevice,
    OUT: embedded_hal::digital::OutputPin,
    IN: embedded_hal::digital::InputPin,
{
    /// Create a new Interface4Pin
    pub fn new(spi: SPI, data_command_pin: OUT, reset_pin: OUT, busy_pin: IN) -> Self {
        Self {
            spi,
            data_command_pin,
            reset_pin,
            busy_pin,
        }
    }

    /// Write data over SPI
    fn write(&mut self, data: &[u8]) -> Result<(), SPI::Error> {
        // Linux has a default limit of 4096 bytes per SPI transfer
        // https://github.com/torvalds/linux/blob/ccda4af0f4b92f7b4c308d3acc262f4a7e3affad/drivers/spi/spidev.c#L93
        if cfg!(target_os = "linux") {
            for data_chunk in data.chunks(4096) {
                self.spi.write(data_chunk)?;
            }
        } else {
            self.spi.write(data)?;
        }

        Ok(())
    }
}

/// Implement the DisplayInterface functions
impl<SPI, OUT, IN> DisplayInterface for Interface4Pin<SPI, OUT, IN>
where
    SPI: embedded_hal::spi::SpiDevice,
    OUT: embedded_hal::digital::OutputPin,
    IN: embedded_hal::digital::InputPin,
{
    type Error = SPI::Error;

    fn reset<D: embedded_hal::delay::DelayNs>(&mut self, delay: &mut D) {
        // Disable the display, the wait for the controller to catch up
        self.reset_pin.set_low().unwrap();
        delay.delay_ms(RESET_DELAY_MS.into());
        // Enable the display, the wait for the controller to catch up
        self.reset_pin.set_high().unwrap();
        delay.delay_ms(RESET_DELAY_MS.into());
    }

    fn send_command(&mut self, command: u8) -> Result<(), Self::Error> {
        // Set the data/command pin as low to indicate command
        self.data_command_pin.set_low().unwrap();
        // Send tthe data
        self.write(&[command])?;
        Ok(())
    }

    fn send_data(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        // Set the data/command pin as high to indicate data
        self.data_command_pin.set_high().unwrap();
        // Send the data
        self.write(data)?;

        Ok(())
    }

    fn busy_wait(&mut self) {
        while match self.busy_pin.is_high() {
            Ok(x) => x,
            _ => false,
        } {}
    }
}
