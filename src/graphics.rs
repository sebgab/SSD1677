use core::usize;

use crate::command::DisplayCommands;
use crate::display::{Display, Rotation};
use crate::interface::DisplayInterface;
use embedded_hal;

#[cfg(feature = "graphics")]
use embedded_graphics_core::{pixelcolor::BinaryColor, prelude::*};

/// A display that holds buffers for drawing into and updating the display.
///
/// When the `graphics` feature is enabled `GraphicsDisplay` implements the `Draw` traif from
/// [embedded-graphics-core](https://crates.io/crates/embedded-graphics-core). This allows
/// for shapes, and text to be rendered on the dsiplay
pub struct GraphicsDisplayBlackAndWhite<'a, I, SPI>
where
    SPI: embedded_hal::spi::SpiDevice,
    I: DisplayInterface + DisplayCommands<SPI>,
{
    display: Display<I, SPI>,
    bw_buffer: &'a mut [u8],
    // TODO: Implement RED support
}

impl<'a, I, SPI> GraphicsDisplayBlackAndWhite<'a, I, SPI>
where
    SPI: embedded_hal::spi::SpiDevice,
    I: DisplayInterface + DisplayCommands<SPI>,
{
    /// Promote a `Display` to a `GraphicsDisplayBlackAndWhite`.
    ///
    /// The B/W buffer musb be provided. It should be `rows` * `cols` / 8 in length.
    pub fn new(display: Display<I, SPI>, bw_buffer: &'a mut [u8]) -> Self {
        GraphicsDisplayBlackAndWhite { display, bw_buffer }
    }

    /// Update the display by writing the buffer to the controller
    pub fn update(&mut self) -> Result<(), <I as DisplayInterface>::Error> {
        self.display.update(Some(self.bw_buffer), None)
    }

    /// DO NOT USE, if this made it into prod it is a mistake...
    // pub fn sebtest(&mut self) -> Result<(), <I as DisplayInterface>::Error> {
    // Fill one and a half complete "line" (y)
    // for i in 0..90 {
    //     // Several lines down
    //     self.bw_buffer.as_mut()[i + 60 * 200] = 0x00;
    // }

    // // Fill one and a half complete "line" (x)
    // for i in 0..150 {
    //     // One hundred lines down
    //     self.bw_buffer.as_mut()[i + 100 * 400] = 0x00;
    // }

    //     self.display.update(Some(self.bw_buffer), None)
    // }

    #[cfg(not(feature = "graphics"))]
    /// Clear the buffer, filling it with black or white depending on the value of `fill_white`.
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
    }

    #[cfg(feature = "graphics")]
    /// Clear the buffer, filling it with a single color given by the `BinaryColor` type.
    pub fn clear(&mut self, color: BinaryColor) {
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
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: BinaryColor) {
        // The display is by default horizontal

        // TODO: Deal with rotation

        // Figure out the index in the buffer to change
        // let buffer_index =
        //     y/8 * self.display.cols() as u32  // Skip the y directon by the number of cols per y , divide by 8 as
        //                                     // there are 8 pixels per byte
        //     + x/8                           // Skip the x direction, divide by 8 as there are 8 pixels
        //                                     // per byte.
        //     ;
        // // Convert the index to usize
        // let buffer_index: usize = buffer_index as usize;

        let (index, bit) = rotation(
            x,
            y,
            self.rows() as u32,
            self.cols() as u32,
            Rotation::Rotate180,
        );
        let index = index as usize;

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

    fn deref(&self) -> &Display<I, SPI> {
        &self.display
    }
}

impl<'a, I, SPI> core::ops::DerefMut for GraphicsDisplayBlackAndWhite<'a, I, SPI>
where
    SPI: embedded_hal::spi::SpiDevice,
    I: DisplayInterface + DisplayCommands<SPI>,
{
    fn deref_mut(&mut self) -> &mut Display<I, SPI> {
        &mut self.display
    }
}

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
        let _ = self.update();

        Ok(())
    }
}

#[cfg(feature = "graphics")]
impl<'a, I, SPI> OriginDimensions for GraphicsDisplayBlackAndWhite<'a, I, SPI>
where
    SPI: embedded_hal::spi::SpiDevice,
    I: DisplayInterface + DisplayCommands<SPI>,
{
    fn size(&self) -> Size {
        // TODO: Handle rotation
        Size::new(self.cols().into(), self.rows().into())
    }
}
