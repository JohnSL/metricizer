// src/main.rs

#![no_std]
#![no_main]

use cortex_m_rt::entry; // The runtime

// use embedded_hal::blocking::i2c;
use hal::{
    delay::Delay,
    i2c::{self, BlockingI2c},
    pac,
    prelude::*,
};
use stm32f1xx_hal as hal; // STM32F1 specific functions

mod keypad;
use keypad::keypad::MyKeypad;

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

    let mut lcd = lcd::Lcd::new(i2c_bus);
    lcd.init(&mut delay);

    loop {
        let f: f32 = 1.2;
        let v = keypad.read(&mut delay);
        if v != 0 {
            delay.delay_ms(f as u16);
        }
        delay.delay_ms(f as u16);
    }
}

mod lcd {
    use hal::{
        delay::Delay,
        prelude::*,
    };
    use stm32f1xx_hal as hal; // STM32F1 specific functions
    use embedded_hal::blocking::i2c;
    
    pub struct Lcd<I>
    where
        I: embedded_hal::blocking::i2c::Write,
    {
        i2c: I,
    }

    impl<I> Lcd<I>
    where
        I: i2c::Write,
    {
        pub fn new(i2c: I) -> Self {
            Lcd { i2c: i2c }
        }

        pub fn init(&mut self, delay: &mut Delay)  -> Result<(), <I as i2c::Write>::Error> {
            delay.delay_ms(50_u16); // Need to wait at least 40ms before sending commands

            self.command(LCD_FUNCTIONSET | LCD_2LINE, delay)?;
            delay.delay_ms(5_u16);

            self.command(LCD_FUNCTIONSET | LCD_2LINE, delay)?;
            delay.delay_ms(5_u16);

            self.command(LCD_FUNCTIONSET | LCD_2LINE, delay);

            self.display_on(delay)
        }

        fn display_on(&mut self, delay: &mut Delay)  -> Result<(), <I as i2c::Write>::Error> {
            let command = LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF;
            self.command(LCD_DISPLAYCONTROL | command, delay)
        }

        // fn command(&mut self, value: u8, delay: &mut Delay) {
        fn command(&mut self, value: u8, delay: &mut Delay) -> Result<(), <I as i2c::Write>::Error> {
            self.send(0x80)?;
            delay.delay_ms(1_u16);

            let result = self.send(value);
            delay.delay_ms(1_u16);

            result
        }

        fn send (&mut self, byte: u8)  -> Result<(), <I as i2c::Write>::Error> {
            self.i2c.write(LCD_ADDRESS, &[byte])
        }
    }

    const LCD_ADDRESS: u8 = 0x7c >> 1;
    // const RGB_ADDRESS: u8 = 0xc0 >> 1;
    const LCD_2LINE: u8 = 0x08;

    // Commands
    const LCD_DISPLAYCONTROL: u8 = 0x08;
    const LCD_FUNCTIONSET: u8 = 0x20;

    // Flags for display on/off control
    const LCD_DISPLAYON: u8 = 0x04;
    const LCD_DISPLAYOFF: u8 = 0x00;
    const LCD_CURSORON: u8 = 0x02;
    const LCD_CURSOROFF: u8 = 0x00;
    const LCD_BLINKON: u8 = 0x01;
    const LCD_BLINKOFF: u8 = 0x00;
    }
