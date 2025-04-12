//! This module provides the [GraphicsDisplayBlackAndWhite] struct for managing
//! a black and white display with graphics capabilities.
//!
//! When the `graphics` feature is enabled, [GraphicsDisplayBlackAndWhite] implements
//! the [DrawTarget] trait from the [embedded-graphics-core] crate, allowing for rendering
//! shapes and text on the display. The struct holds a buffer for drawing and updating
//! the display contents.
//!
//! # Features
//!
//! - **Graphics Support**: When the `graphics` feature is enabled, additional methods
//!   for drawing graphics and handling colors are available.
//!
//! [DrawTarget]: https://docs.rs/embedded-graphics-core/0.4.0/embedded_graphics_core/draw_target/trait.DrawTarget.html
//! [embedded-graphics-core]: https://crates.io/crates/embedded-graphics-core
use crate::command::DisplayCommands;
use crate::display::{Display, DisplayUpdateMode, Rotation};
use crate::interface::DisplayInterface;
use core::usize;
use embedded_hal;

#[cfg(feature = "graphics")]
use embedded_graphics_core::{pixelcolor::BinaryColor, prelude::*};

/// A display that holds buffers for drawing into and updating the display.
pub struct GraphicsDisplayBlackAndWhite<'a, I, SPI>
where
    SPI: embedded_hal::spi::SpiDevice,
    I: DisplayInterface + DisplayCommands<SPI>,
{
    display: Display<I, SPI>, // The underlying display interface
    bw_buffer: &'a mut [u8],  // The buffer for black and white pixel data
                              // TODO: Implement RED support
}

impl<'a, I, SPI> GraphicsDisplayBlackAndWhite<'a, I, SPI>
where
    SPI: embedded_hal::spi::SpiDevice,
    I: DisplayInterface + DisplayCommands<SPI>,
{
    /// Promote a [Display] to a [GraphicsDisplayBlackAndWhite].
    ///
    /// The black and white buffer must be provided. It should be of length
    /// `rows * cols / 8`, where `rows` and `cols` are the dimensions of the display.
    ///
    /// # Arguments
    ///
    /// * `display` - The underlying display instance.
    /// * `bw_buffer` - A mutable reference to the buffer for black and white pixel data.
    pub fn new(display: Display<I, SPI>, bw_buffer: &'a mut [u8]) -> Self {
        GraphicsDisplayBlackAndWhite { display, bw_buffer }
    }

    /// Update the display by writing the buffer to the controller.
    ///
    /// This method sends the current buffer to the display controller to refresh
    /// the display contents.
    ///
    /// # Arguments
    ///
    /// * `mode` - The kind of update to perform, see [DisplayUpdateMode] for details.
    ///
    /// # Returns
    ///
    /// * `Result<(), <I as DisplayInterface>::Error>` - Returns `Ok(())` on success,
    ///   or an error if the update fails.
    pub fn update(
        &mut self,
        mode: DisplayUpdateMode,
    ) -> Result<(), <I as DisplayInterface>::Error> {
        self.display.update(Some(self.bw_buffer), None, mode)
    }

    #[cfg(not(feature = "graphics"))]
    /// Clear the buffer, filling it with black or white depending on the value of `fill_white`.
    ///
    /// # Arguments
    ///
    /// * `fill_white` - If `true`, the buffer is filled with white; otherwise, it is filled with black.
    pub fn clear(&mut self, fill_white: bool) {
        // Figure out the fill value
        let fill_value: u8 = match fill_white {
            true => 0xFF,
            false => 0x00,
        };

        // Loop through the buffer
        for byte in &mut self.bw_buffer.as_mut().iter_mut() {
            // Set the value of the byte
            *byte = fill_value;
        }

        // Refresh the display
        self.update(DisplayUpdateMode::Slow)
    }

    #[cfg(feature = "graphics")]
    /// Clear the buffer, filling it with a single color given by the [BinaryColor] type.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to fill the buffer with, represented as a [BinaryColor].
    ///
    /// [BinaryColor]: https://docs.rs/embedded-graphics-core/0.4.0/embedded_graphics_core/pixelcolor/enum.BinaryColor.html
    pub fn clear(&mut self, color: BinaryColor) -> Result<(), <I as DisplayInterface>::Error> {
        // Figure out the fill value
        let fill_value: u8 = match color {
            BinaryColor::On => 0x00,
            BinaryColor::Off => 0xFF,
        };

        // Loop through the buffer
        for byte in &mut self.bw_buffer.as_mut().iter_mut() {
            // Set the value of the byte
            *byte = fill_value;
        }

        // Refresh the display
        self.update(DisplayUpdateMode::Slow)
    }

    /// Set a pixel at the specified coordinates to the given color.
    ///
    /// This method updates the buffer to reflect the color of the pixel at the
    /// specified `(x, y)` coordinates, taking into account the current rotation
    /// of the display.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the pixel.
    /// * `y` - The y-coordinate of the pixel.
    /// * `color` - The color to set the pixel to, represented as a [BinaryColor].
    ///
    /// [BinaryColor]: https://docs.rs/embedded-graphics-core/0.4.0/embedded_graphics_core/pixelcolor/enum.BinaryColor.html
    pub fn set_pixel(&mut self, x: u32, y: u32, color: BinaryColor) {
        // Calculate the value of x depending on the rotation
        // TODO: Move into the rotation function
        let x = match self.rotation() {
            Rotation::Rotate0 | Rotation::Rotate180 => self.cols() as u32 - x,
            Rotation::Rotate90 | Rotation::Rotate270 => self.rows() as u32 - x,
        };

        // Find out the buffer index and bit value
        let (index, bit) = rotation(
            x,
            y,
            self.cols() as u32,
            self.rows() as u32,
            self.rotation(),
        );
        let index = index as usize;

        // Set the value in the display buffer
        match color {
            BinaryColor::On => {
                self.bw_buffer.as_mut()[index] &= !bit;
            }
            BinaryColor::Off => {
                self.bw_buffer.as_mut()[index] |= bit;
            }
        }
    }
}

impl<'a, I, SPI> core::ops::Deref for GraphicsDisplayBlackAndWhite<'a, I, SPI>
where
    SPI: embedded_hal::spi::SpiDevice,
    I: DisplayInterface + DisplayCommands<SPI>,
{
    type Target = Display<I, SPI>;

    /// Dereference to access the underlying [Display] instance.
    ///
    /// This allows for direct access to the methods and properties of the
    /// [Display] struct.
    fn deref(&self) -> &Display<I, SPI> {
        &self.display
    }
}

impl<'a, I, SPI> core::ops::DerefMut for GraphicsDisplayBlackAndWhite<'a, I, SPI>
where
    SPI: embedded_hal::spi::SpiDevice,
    I: DisplayInterface + DisplayCommands<SPI>,
{
    /// Mutably dereference to access the underlying [Display] instance.
    ///
    /// This allows for modification of the [Display] struct and its properties.
    fn deref_mut(&mut self) -> &mut Display<I, SPI> {
        &mut self.display
    }
}

/// Calculate the pixel index and bit mask for a given pixel position based on the rotation.
///
/// This function determines the appropriate index in the buffer and the bit mask
/// for the specified `(x, y)` coordinates, taking into account the current rotation
/// of the display.
///
/// # Arguments
///
/// * `x` - The x-coordinate of the pixel.
/// * `y` - The y-coordinate of the pixel.
/// * `width` - The width of the display in pixels.
/// * `height` - The height of the display in pixels.
/// * `rotation` - The current rotation of the display.
///
/// # Returns
///
/// * `(u32, u8)` - A tuple containing the index in the buffer and the bit mask for the pixel.
fn rotation(x: u32, y: u32, width: u32, height: u32, rotation: Rotation) -> (u32, u8) {
    match rotation {
        Rotation::Rotate0 => (x / 8 + (width / 8) * y, 0x80 >> (x % 8)),
        Rotation::Rotate90 => ((width - 1 - y) / 8 + (width / 8) * x, 0x01 << (y % 8)),
        Rotation::Rotate180 => (
            ((width / 8) * height - 1) - (x / 8 + (width / 8) * y),
            0x01 << (x % 8),
        ),
        Rotation::Rotate270 => (y / 8 + (height - 1 - x) * (width / 8), 0x80 >> (y % 8)),
    }
}

#[cfg(feature = "graphics")]
impl<'a, I, SPI> DrawTarget for GraphicsDisplayBlackAndWhite<'a, I, SPI>
where
    SPI: embedded_hal::spi::SpiDevice,
    I: DisplayInterface + DisplayCommands<SPI>,
{
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    /// Draw pixels from an iterator onto the display.
    ///
    /// This method takes an iterator of [Pixel] items and sets the corresponding
    /// pixels in the display buffer. After drawing, it updates the display to
    /// reflect the changes.
    ///
    /// # Arguments
    ///
    /// * `pixels` - An iterator of [`Pixel<Self::Color>`][Pixel] items to draw on the display.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - Always returns `Ok(())` since the error type is infallible.
    ///   This indicates that the drawing operation cannot fail.
    ///
    /// [Pixel]: https://docs.rs/embedded-graphics-core/0.4.0/embedded_graphics_core/struct.Pixel.html
    fn draw_iter<Iter>(&mut self, pixels: Iter) -> Result<(), Self::Error>
    where
        Iter: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let size = self.size();

        // Draw the image pixel by pixel
        for Pixel(Point { x, y }, color) in pixels {
            let x = x as u32;
            let y = y as u32;

            if x < size.width && y < size.height {
                self.set_pixel(x, y, color);
            }
        }

        // Refresh the display, ignoring any errors
        // TODO: Handle errors
        let _ = self.update(DisplayUpdateMode::Fast);

        Ok(())
    }
}

#[cfg(feature = "graphics")]
impl<'a, I, SPI> OriginDimensions for GraphicsDisplayBlackAndWhite<'a, I, SPI>
where
    SPI: embedded_hal::spi::SpiDevice,
    I: DisplayInterface + DisplayCommands<SPI>,
{
    /// Get the size of the display in pixels.
    ///
    /// This method returns the dimensions of the display based on its current
    /// rotation. The size is represented as a [Size] struct, which contains
    /// the width and height of the display.
    ///
    /// # Returns
    ///
    /// * [`Size`] - The dimensions of the display in pixels.
    fn size(&self) -> Size {
        match self.rotation() {
            Rotation::Rotate0 | Rotation::Rotate180 => {
                Size::new(self.cols().into(), self.rows().into())
            }
            Rotation::Rotate90 | Rotation::Rotate270 => {
                Size::new(self.rows().into(), self.cols().into())
            }
        }
    }
}
