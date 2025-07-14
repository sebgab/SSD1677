# SSD1677 driver

Blocking SPI driver to use SSD1677 e-paper displays in embedded Rust.  
The driver currently supports binary monochromatic usage.

This driver is tested on the [GDEQ0426T82 display from GoodDisplay](https://www.good-display.com/product/457.html).

## Features

Implemented:
- Binary Monochromatic support
- [Embedded-Graphics](https://crates.io/crates/embedded-graphics) support

Not implemented:
- Red support
- Async

## Usage

The following section will show a simplified example based on the example in the `exmaples` directory.
This snippet will not compile by itself and is only intended as a basic example of how the display can be initialised.

```rust
use ssd1677::{self, interface::Interface4Pin};

fn main() -> ! {
    // Initialize the device pins
    let dc = Output::new(dc_pin, gpio::Level::Low, gpio::Speed::Medium);
    let reset = Output::new(reset_pin, gpio::Level::High, gpio::Speed::Medium);
    let busy = Input::new(busy_pin, gpio::Pull::None);

    // Create the display configuration
    let config: ssd1677::Config = ssd1677::ConfigBuilder::new()
        .dimensions(ssd1677::Dimensions {
            rows: 480,
            cols: 800,
        })
        .rotation(ssd1677::Rotation::Rotate0)
        .auto_update(false)
        .build()
        .expect("Failed to create display config");

    // Create the display interface
    let interface = Interface4Pin::new(display_spi_device, dc, reset, busy);

    // Create the pixel buffer for the display.
    // This needs to be large enough to store the entire display contents.
    // One bit per pixel is used for a black-and-white display.
    let mut display_buffer = [0u8; 480 * 800 / 8];

    // Create the display
    let mut display = ssd1677::Display::new(interface, &mut display_buffer, config);

    // Reset the display so it is ready for use
    display.reset(&mut Delay).expect("Failed to reset display");

    // You now have an initialised display, you can now draw to 
    // the display using embedded graphics.
}
```
