#![no_std]
#![no_main]

use core::{cell::RefCell, convert::Infallible};
#[allow(unused_imports)]
use defmt::{debug, error, info, trace, warn};
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_stm32::{
    Peri,
    gpio::{self, AnyPin, Input, Level, Output},
    spi::{self, Spi},
    time::Hertz,
};
use embassy_sync::blocking_mutex::{NoopMutex, raw::NoopRawMutex};
use embassy_time::{Delay, Timer};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{
        Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
    },
    text::{Alignment, Text},
};
use embedded_graphics_core;
use ssd1677::{self, interface::Interface4Pin};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// Global to store the peripheral SPI bus
static SPI_BUS: StaticCell<NoopMutex<RefCell<Spi<embassy_stm32::mode::Blocking>>>> =
    StaticCell::new();

#[embassy_executor::task]
pub async fn gui_task(
    display_spi_device: SpiDeviceWithConfig<
        'static,
        NoopRawMutex,
        Spi<'static, embassy_stm32::mode::Blocking>,
        gpio::Output<'static>,
    >,
    reset_pin: Peri<'static, AnyPin>,
    dc_pin: Peri<'static, AnyPin>,
    busy_pin: Peri<'static, AnyPin>,
) -> ! {
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
        .rotation(ssd1677::Rotation::Rotate270)
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

    info!("Initialised display");

    draw_embedded_graphics_demo(&mut display).unwrap();

    info!("Drew demo");

    loop {
        // TODO: Make loop do something cool
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    /////////////////////
    // Configure STM32 //
    /////////////////////

    // Create a config for the STM32 with default values
    let mut config = embassy_stm32::Config::default();
    // Enable the internal 16MHz clock
    config.rcc.hsi = true;
    // Enable PLL1 and configure it to be 160MHz, highly recommended so the display
    // refreshes at an acceptable rate. Embassy auto-sleeps the device when idle, as
    // such power draw from this is not a big concern.
    config.rcc.pll1 = Some({
        use embassy_stm32::rcc::*;

        Pll {
            source: PllSource::HSI,  // 16MHz
            prediv: PllPreDiv::DIV1, // No div
            mul: PllMul::MUL10,      // 16MHz * 10 = 160MHz
            divp: None,
            divq: None,
            divr: Some(PllDiv::DIV1),
        }
    });
    // Configure the system to use the internal high speed clock source
    config.rcc.sys = embassy_stm32::rcc::Sysclk::PLL1_R;

    let p = embassy_stm32::init(config);

    info!("Initialised STM");

    /////////////////////
    // Initialize SPI  //
    /////////////////////

    // Configure the pins
    let sck = p.PA5;
    let mosi = p.PA7;
    let miso = p.PA6;

    // Create the SPI peripheral
    let spi = Spi::new_blocking(p.SPI1, sck, mosi, miso, spi::Config::default());

    // Store the SPI peripheral in the global static
    let spi_bus = NoopMutex::new(RefCell::new(spi));
    let spi_bus = SPI_BUS.init(spi_bus);

    info!("Initialised SPI");

    ///////////////////////
    // Setup for Display //
    ///////////////////////

    // Create the SPI config to use with the display
    let mut display_spi_config = spi::Config::default();
    display_spi_config.frequency = Hertz(10_000_000);
    display_spi_config.mode = spi::MODE_0;

    // Create the display SPI device
    let display_cs = Output::new(p.PA8, Level::High, embassy_stm32::gpio::Speed::Medium);
    let display_spi_device = SpiDeviceWithConfig::new(spi_bus, display_cs, display_spi_config);

    // Spawn the GUI task
    info!("Spawning task: gui_task");
    spawner
        .spawn(gui_task(
            display_spi_device,
            p.PA11.into(),
            p.PA10.into(),
            p.PA12.into(),
        ))
        .expect("Failed to spawn task: gui_task");

    loop {
        Timer::after_secs(5).await;
        info!("Ping!")
    }
}

fn draw_embedded_graphics_demo<D>(display: &mut D) -> Result<(), core::convert::Infallible>
where
    D: embedded_graphics_core::draw_target::DrawTarget<Color = BinaryColor>,
    Infallible: From<<D as DrawTarget>::Error>,
{
    // Create styles used by the drawing operations.
    let thin_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    let thick_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 3);
    let border_stroke = PrimitiveStyleBuilder::new()
        .stroke_color(BinaryColor::On)
        .stroke_width(3)
        .stroke_alignment(StrokeAlignment::Inside)
        .build();
    let fill = PrimitiveStyle::with_fill(BinaryColor::On);
    let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    let yoffset = 10;

    // Draw a 3px wide outline around the display.
    display
        .bounding_box()
        .into_styled(border_stroke)
        .draw(display)?;

    // Draw a triangle.
    Triangle::new(
        Point::new(16, 16 + yoffset),
        Point::new(16 + 16, 16 + yoffset),
        Point::new(16 + 8, yoffset),
    )
    .into_styled(thin_stroke)
    .draw(display)?;

    // Draw a filled square
    Rectangle::new(Point::new(52, yoffset), Size::new(16, 16))
        .into_styled(fill)
        .draw(display)?;

    // Draw a circle with a 3px wide stroke.
    Circle::new(Point::new(88, yoffset), 17)
        .into_styled(thick_stroke)
        .draw(display)?;

    // Draw centered text.
    let text = "embedded-graphics";
    Text::with_alignment(
        text,
        display.bounding_box().center() + Point::new(0, 15),
        character_style,
        Alignment::Center,
    )
    .draw(display)?;

    Ok(())
}
