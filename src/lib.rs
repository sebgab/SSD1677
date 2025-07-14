#![no_std]

//! SSD1677 ePaper display driver.  
//! This driver is heavily inspired by the [SSD1675 driver by wezm](https://github.com/wezm/ssd1675).
//!
//! ## Usage
//!
//!
//! ### Creation and initialization
//! To control a display you will need:
//! * An [Interface] to the controller
//! * A [display configuration][Config]
//! * A [BasicDisplay]
//!
//! The [Interface] has the hardware connection details to the SSD1677 controller. This crate only
//! implements the 4-wire SPI communication, for this a SPI device and some GPIO pins are required.
//!
//! To configure the details of your specific display create a [Config]. This contains information
//! about the size of the display, and the display rotation.  
//! To construct the Config use the [Builder] interface.
//!
//! The SSD1677 controller can control many different displays of varying sizes and color
//! capabilities. This driver should work on any size display the controller can do, the underlying
//! functions for supporting red color are implemented in the driver, but the [Display]
//! used to support [embedded-graphics] does not implement it at the current time.
//!
//! Using the [Config] from erlier create a [BasicDisplay] instance.
//! The [BasicDisplay] only has basic buffer rendering capabilities.
//!
//! The [BasicDisplay] can optionally be updated with [embedded-graphics] by promoting it to a
//! [Display].
//!
//! [Interface]: interface/struct.Interface4Pin.html
//! [Config]: config/struct.Config.html
//! [BasicDisplay]: basic_display/struct.BasicDisplay.html
//! [Display]: display/struct.Display.html
//! [embedded-graphics]: https://crates.io/crates/embedded-graphics
//! [Builder]: confg/struct.Builder.html

pub mod basic_display;
pub mod command;
pub mod config;
pub mod display;
pub mod error;
pub mod interface;

pub use basic_display::{Dimensions, Rotation};
pub use config::{Builder as ConfigBuilder, Config};
pub use display::Display;
