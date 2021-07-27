// src/main.rs

#![no_std]
#![no_main]

use cortex_m_rt::entry; // The runtime

use hal::{
    delay::Delay,
    i2c::{self, BlockingI2c},
    pac,
    prelude::*,
};
use stm32f1xx_hal as hal; // STM32F1 specific functions

mod keypad;
use keypad::keypad::MyKeypad;

mod display;
use display::display::Lcd;

#[allow(unused_imports)]
// use panic_halt; // When a panic occurs, stop the microcontroller
use panic_semihosting;

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();
    let core_peripherals = cortex_m::Peripherals::take().unwrap();

    let mut rcc = peripherals.RCC.constrain();

    let mut gpioa = peripherals.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = peripherals.GPIOB.split(&mut rcc.apb2);
    let mut gpioc = peripherals.GPIOC.split(&mut rcc.apb2);

    let mut flash = peripherals.FLASH.constrain();

    let clocks = rcc.cfgr.sysclk(8.mhz()).freeze(&mut flash.acr);

    let mut delay = Delay::new(core_peripherals.SYST, clocks);

    // Set up the keypad
    let rows = (
        gpiob.pb15.into_pull_up_input(&mut gpiob.crh),
        gpioa.pa7.into_pull_up_input(&mut gpioa.crl),
        gpiob.pb6.into_pull_up_input(&mut gpiob.crl),
        gpioa.pa9.into_pull_up_input(&mut gpioa.crh),
    );

    let cols = (
        gpioa.pa8.into_open_drain_output(&mut gpioa.crh),
        gpiob.pb5.into_open_drain_output(&mut gpiob.crl),
        gpioc.pc7.into_open_drain_output(&mut gpioc.crl),
    );

    let mut keypad = MyKeypad::new(rows, cols);

    // Now setup the LCD display
    let scl = gpiob.pb10.into_alternate_open_drain(&mut gpiob.crh);
    let sda = gpiob.pb11.into_alternate_open_drain(&mut gpiob.crh);

    let i2c_bus = BlockingI2c::i2c2(
        peripherals.I2C2,
        (scl, sda),
        i2c::Mode::Standard {
            frequency: 400_000.hz(),
        },
        clocks,
        &mut rcc.apb1,
        1000,
        10,
        1000,
        1000,
    );

    let lcd = Lcd::new(i2c_bus);
    let mut app = metricizer::MainApp::new(lcd, &mut delay).unwrap();
    app.clear().unwrap();

    loop {
        let raw_key = keypad.read(&mut delay);
        if raw_key != 0 {
            let digit = keypad.convert(raw_key);
            app.digit(digit).unwrap();
            while keypad.read(&mut delay) != 0 {}
        }
    }
}

mod metricizer {
    use crate::display::display::Lcd;
    use embedded_hal::blocking::{i2c, delay::DelayMs};
    use core::{fmt::Write, u8};
    use heapless::String;

    const STAR_KEY: i16 = -1;
    const HASH_KEY: i16 = -2;
    
        pub struct MainApp<I>
    where
        I: i2c::Write
    {
        lcd: Lcd<I>,
        line1: String<16>,
        line2: String<16>,
        input: f32,
        found_dot: bool
}

    impl<I> MainApp<I>
    where
        I: i2c::Write
    {
        pub fn new(lcd: Lcd<I>, delay: &mut dyn DelayMs<u16>) -> Result<MainApp<I>, <I as i2c::Write>::Error>
        where
            I: i2c::Write
        {
            let mut app = MainApp {
                lcd: lcd,
                line1: String::new(),
                line2: String::new(),
                input: 0.,
                found_dot: false
            };
            app.init(delay)?;
            Ok(app)
        }

        fn init(&mut self, delay: &mut dyn DelayMs<u16>) -> Result<(), <I as i2c::Write>::Error> {
            self.lcd.init(delay)?;
            self.lcd.init(delay)?;
            self.lcd.cursor_on()
        }

        pub fn clear(&mut self) -> Result<(), <I as i2c::Write>::Error> {
            self.digit(STAR_KEY)
        }

        pub fn digit(&mut self, digit: i16) -> Result<(), <I as i2c::Write>::Error> {

            match digit {
                STAR_KEY => self.input = 0.,
                _ => self.input = 10. * self.input + digit as f32
            }
            self.update()
        }

        pub fn update(&mut self) -> Result<(), <I as i2c::Write>::Error> {
            // Add the value typed in so far to the line
            self.line1.clear();
            let _ = write!(self.line1, "{}", self.input);
            let cursor = if self.input > 0. {
                self.line1.len() as u8
            } else {
                0
            };

            self.line2.clear();
            self.line2.push_str(&self.line1).unwrap();

            self.lcd.cursor_position(0, 0)?;
            // self.lcd.print(".01\"    0.254mm")?;
            pad_line(&mut self.line1);
            self.lcd.print(&self.line1)?;

            self.lcd.cursor_position(0, 1)?;
            pad_line(&mut self.line2);
            self.lcd.print(&self.line2)?;
            // self.lcd.print(".01mm   .00039\"")?;

            self.lcd.cursor_position(cursor, 0)
        }
    }

    // Adds spaces to the end of the line, which effectively clears the "right" side of the line.
    fn pad_line(line: &mut String<16>) {
        for _ in 0..line.capacity()-line.len() {
            line.push(' ').unwrap();
        }
    }
}
