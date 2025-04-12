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
//! * A [Display]
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
//! functions for supporting red color are implemented in the driver, but the [GraphicsDisplay]
//! used to support [embedded-graphics] does not implement it at the current time.
//!
//! Using the [Config] from erlier create a [Display] instance.
//! The [Display] only has basic buffer rendering capabilities.
//!
//! The [Display] can optionally be updated with [embedded-graphics] by promoting it to a
//! [GraphicsDisplay].
//!
//! [Interface]: interface/struct.Interface4Pin.html
//! [Config]: config/struct.Config.html
//! [Display]: display/struct.Display.html
//! [GraphicsDisplay]: graphics/struct.GraphicsDisplayBlackAndWhite.html
//! [embedded-graphics]: https://crates.io/crates/embedded-graphics

pub mod command;
pub mod config;
pub mod display;
pub mod error;
pub mod graphics;
pub mod interface;
