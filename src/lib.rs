#![no_std]

pub mod command;
pub mod config;
pub mod display;
pub mod error;
pub mod graphics;
pub mod interface;

use crate::interface::Interface4Pin;
impl<SPI, OUT, IN> Interface4Pin<SPI, OUT, IN>
where
    SPI: embedded_hal::spi::SpiDevice,
    OUT: embedded_hal::digital::OutputPin,
    IN: embedded_hal::digital::InputPin,
{
}
