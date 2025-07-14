#![no_std]

//! SSD1677 e-paper display driver.  
//! The structure of this driver is heavily inspired by the [SSD1675 driver by wezm](https://github.com/wezm/ssd1675).
//!
//! ## Usage
//!
//! This driver exposes control over the display in three tiers of complexity.
//! From lowest-to-highest level they are:  
//! * [`Interface`] to directly control the SSD1677 display controller
//! * [`BasicDisplay`] to draw to the display manually
//! * [`Display`] to use [embedded-graphics]
//!
//! For basic usage with [embedded-graphics] use [`Display`].  
//! Embedded-graphics support can be removed by not using the `graphics` feature flag.
//!
//! ### Creation and initialization
//!
//! The [`Interface`] has the hardware connection details to the SSD1677 controller. This crate only
//! implements the 4-wire SPI communication, for this a SPI device and some GPIO pins are required.
//!
//! To configure the details of your specific display create a [`Config`]. This contains information
//! about the size of the display, and the display rotation.  
//! To construct the Config use the [Builder] interface.
//!
//! The SSD1677 controller can control many different displays of varying sizes and color
//! capabilities. This driver should work on any size display the controller can do, the underlying
//! functions for supporting red color are implemented in the driver, but [`Display`]
//! used to support [embedded-graphics] does not implement it at the current time.
//!
//! Lastly create a [`Display`] with the [`Config`].
//! The display must be reset before use.
//!
//!
//! #### Example
//! The following example is a snippet from the example in the `examples` folder of the repository.
//! This snippet of the example does not compile on it's own, but demonstrates a basic implementation of the display driver.
//!
//! ```rust
//! use ssd1677::{self, interface::Interface4Pin};
//!
//! fn main() -> ! {
//!     // Initialize the device pins
//!     let dc = Output::new(dc_pin, gpio::Level::Low, gpio::Speed::Medium);
//!     let reset = Output::new(reset_pin, gpio::Level::High, gpio::Speed::Medium);
//!     let busy = Input::new(busy_pin, gpio::Pull::None);
//!     
//!     // Create the display configuration
//!     let config: ssd1677::Config = ssd1677::ConfigBuilder::new()
//!         .dimensions(ssd1677::Dimensions {
//!             rows: 480,
//!             cols: 800,
//!         })
//!         .rotation(ssd1677::Rotation::Rotate0)
//!         .auto_update(false)
//!         .build()
//!         .expect("Failed to create display config");
//!     
//!     // Create the display interface
//!     let interface = Interface4Pin::new(display_spi_device, dc, reset, busy);
//!     
//!     // Create the pixel buffer for the display.
//!     // This needs to be large enough to store the entire display contents.
//!     // One bit per pixel is used for a black-and-white display.
//!     let mut display_buffer = [0u8; 480 * 800 / 8];
//!     
//!     // Create the display
//!     let mut display = ssd1677::Display::new(interface, &mut display_buffer, config);
//!     
//!     // Reset the display so it is ready for use
//!     display.reset(&mut Delay).expect("Failed to reset display");
//!     
//!     // You now have an initialised display, you can now draw to
//!     // the display using embedded graphics.
//! }
//! ```
//!
//! [`Interface`]: interface/struct.Interface4Pin.html
//! [`BasicDisplay`]: basic_display/struct.BasicDisplay.html
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
