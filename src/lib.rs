#![no_std]

mod command;
pub mod config;
mod display;
mod error;
mod interface;

pub use command::*;
pub use display::*;
pub use interface::DisplayInterface;
pub use interface::Interface4Pin;

impl<SPI, OUT, IN> Interface4Pin<SPI, OUT, IN>
where
    SPI: embedded_hal::spi::SpiDevice,
    OUT: embedded_hal::digital::OutputPin,
    IN: embedded_hal::digital::InputPin,
{
}
