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

mod metricizer;

use lcd_1602_i2c::Lcd;

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

    let lcd = Lcd::new(i2c_bus, &mut delay).unwrap();
    let mut app = metricizer::MainApp::new(lcd).unwrap();
    app.clear().unwrap();

    loop {
        let key = keypad.read_char(&mut delay);
        if key != ' ' {
            app.key(key).unwrap();
            while keypad.read_char(&mut delay) != ' ' {}
        }
    }
}
