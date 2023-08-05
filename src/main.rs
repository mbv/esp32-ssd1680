use esp_idf_hal::delay::{Ets, FreeRtos};
use esp_idf_hal::gpio::{AnyIOPin, PinDriver};
use esp_idf_hal::spi::{SpiDeviceDriver, SpiDriver, SpiDriverConfig, config::Config};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;
use ssd1680::prelude::*;
use ssd1680::color::{Black, White, Red};

use embedded_graphics::{
    fonts::{Font6x8, Text},
    prelude::*,
    primitives::{Circle, Line, Rectangle},
    style::PrimitiveStyle,
    text_style,
};

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let spi = peripherals.spi2;

    let rst = PinDriver::output(peripherals.pins.gpio18).unwrap();
    let dc = PinDriver::output(peripherals.pins.gpio10).unwrap();
    let busy = PinDriver::input(peripherals.pins.gpio19).unwrap();
    let mut delay = Ets;

    let sclk = peripherals.pins.gpio6;
    let sdo = peripherals.pins.gpio7;


    let spi = SpiDriver::new(
        spi,
        sclk,
        sdo,
        None::<AnyIOPin>,
        &SpiDriverConfig::default()
    ).unwrap();

    let cs = PinDriver::output(peripherals.pins.gpio9).unwrap();

    let mut spi = SpiDeviceDriver::new(spi, Option::<AnyIOPin>::None, &Config::new()).unwrap();

    // Initialise display controller
    let mut ssd1680 = Ssd1680::new(&mut spi, cs, busy, dc, rst, &mut delay).unwrap();

    // Clear frames on the display driver
    ssd1680.clear_red_frame(&mut spi).unwrap();
    ssd1680.clear_bw_frame(&mut spi).unwrap();

    // Create buffer for black and white
    let mut display_bw = Display2in13::bw();

    draw_rotation_and_rulers(&mut display_bw);

    display_bw.set_rotation(DisplayRotation::Rotate0);
    Rectangle::new(Point::new(60, 60), Point::new(100, 100))
        .into_styled(PrimitiveStyle::with_fill(Black))
        .draw(&mut display_bw)
        .unwrap();

    info!("Send bw frame to display");
    ssd1680.update_bw_frame(&mut spi, display_bw.buffer()).unwrap();

    // Draw red color
    let mut display_red = Display2in13::red();

    Circle::new(Point::new(100, 100), 20)
        .into_styled(PrimitiveStyle::with_fill(Red))
        .draw(&mut display_red)
        .unwrap();

    // println!("Send red frame to display");
    ssd1680.update_red_frame(&mut spi, display_red.buffer()).unwrap();

    info!("Update display");
    ssd1680.display_frame(&mut spi, &mut FreeRtos).unwrap();

    info!("Done");
    loop {
        FreeRtos::delay_ms(1000)
    }
}

fn draw_rotation_and_rulers(display: &mut Display2in13) {
    display.set_rotation(DisplayRotation::Rotate0);
    draw_text(display, "rotation 0", 25, 25);
    draw_ruler(display);

    display.set_rotation(DisplayRotation::Rotate90);
    draw_text(display, "rotation 90", 25, 25);
    draw_ruler(display);

    display.set_rotation(DisplayRotation::Rotate180);
    draw_text(display, "rotation 180", 25, 25);
    draw_ruler(display);

    display.set_rotation(DisplayRotation::Rotate270);
    draw_text(display, "rotation 270", 25, 25);
    draw_ruler(display);
}

fn draw_ruler(display: &mut Display2in13) {
    for col in 1..ssd1680::WIDTH {
        if col % 25 == 0 {
            Line::new(Point::new(col as i32, 0), Point::new(col as i32, 10))
                .into_styled(PrimitiveStyle::with_stroke(Black, 1))
                .draw(display)
                .unwrap();
        }

        if col % 50 == 0 {
            let label = col.to_string();
            draw_text(display, &label, col as i32, 12);
        }
    }
}

fn draw_text(display: &mut Display2in13, text: &str, x: i32, y: i32) {
    let _ = Text::new(text, Point::new(x, y))
        .into_styled(text_style!(
            font = Font6x8,
            text_color = Black,
            background_color = White
        ))
        .draw(display);
}

