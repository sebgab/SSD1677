#![no_std]
#![no_main]

use core::cell::RefCell;
use defmt::Format;
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
use embassy_time::{Delay, Duration, Instant, Timer};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};
use heapless::String;
use ssd1677::{self, interface::Interface4Pin};
use static_cell::StaticCell;
use u8g2_fonts::{U8g2TextStyle, fonts};
use {defmt_rtt as _, panic_probe as _};

// Global to store the peripheral SPI bus
static SPI_BUS: StaticCell<NoopMutex<RefCell<Spi<embassy_stm32::mode::Blocking>>>> =
    StaticCell::new();

#[derive(Copy, Clone, Debug, Eq, PartialEq, Format)]
struct TimeTimer {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub days: u8,
}

impl TimeTimer {
    pub fn new() -> Self {
        TimeTimer {
            hours: 0,
            minutes: 0,
            seconds: 0,
            days: 0,
        }
    }

    pub fn increment(&mut self) {
        // Increment the second
        self.seconds += 1;

        // If we are within the confines of a second, return
        if self.seconds < 60 {
            return;
        }

        // New minute
        self.minutes += 1;
        self.seconds = 0;

        // If we are within the confines of an hour, return
        if self.minutes < 60 {
            return;
        }

        // New hour
        self.hours += 1;
        self.minutes = 0;

        // If we are within confines of a day
        if self.hours < 24 {
            return;
        }

        // New day
        self.days += 1;
        self.hours = 0;
    }
}

impl Drawable for TimeTimer {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        // Format the string
        let text: String<16> = heapless::format!(
            "{:03}:{:02}:{:02}:{:02}",
            self.days,
            self.hours,
            self.minutes,
            self.seconds
        )
        .expect("Failed to format string");

        // Create the text style
        let character_style =
            U8g2TextStyle::new(fonts::u8g2_font_7Segments_26x42_mn, BinaryColor::On);

        // Draw the string

        Text::with_alignment(
            text.as_str(),
            target.bounding_box().center(),
            character_style,
            Alignment::Center,
        )
        .draw(target)?;

        // All OK
        Ok(())
    }
}

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
        .rotation(ssd1677::Rotation::Rotate180)
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

    // Clear the display
    display.clear(BinaryColor::Off).unwrap();
    display
        .update(ssd1677::basic_display::DisplayUpdateMode::Slow)
        .unwrap();

    info!("Cleared display");

    // Create the counter
    let mut counter = TimeTimer::new();

    // Last update
    let mut prev_update = Instant::now();

    loop {
        // Check if we should increment the timer, delay appropriately
        let time_since_last_update = Instant::now() - prev_update;
        if time_since_last_update < Duration::from_secs(1) {
            // It is less than a second since the last update, sleep the remainder
            Timer::after(Duration::from_secs(1) - time_since_last_update).await;
        }

        // New update should occur
        prev_update = Instant::now();

        // Increment the counter
        counter.increment();
        debug!("{}", counter);

        // Draw the counter to the display buffer, make sure to clear the display first
        display.clear(BinaryColor::Off).unwrap();
        counter.draw(&mut display);

        // Update the display contents
        display
            .update(ssd1677::basic_display::DisplayUpdateMode::Fast)
            .unwrap();
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
