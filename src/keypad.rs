use stm32f1xx_hal as hal;
use hal::gpio::{Input, OpenDrain, Output, PullUp};
use hal::gpio::gpioa::{PA7, PA8, PA9};
use hal::gpio::gpiob::{PB5, PB6, PB10};
use hal::gpio::gpioc::{PC7};

pub type KeypadRows = (
    PB10<Input<PullUp>>,
    PA7<Input<PullUp>>,
    PB6<Input<PullUp>>,
    PA9<Input<PullUp>>,
);

pub type KeypadColumns = (
    PA8<Output<OpenDrain>>,
    PB5<Output<OpenDrain>>,
    PC7<Output<OpenDrain>>,
);

pub mod keypad {
    use embedded_hal::digital::v2::{InputPin, OutputPin};
    use stm32f1xx_hal::{delay::Delay, prelude::*};

    pub struct MyKeypad {
        rows: super::KeypadRows,
        columns: super::KeypadColumns,
    }

    impl MyKeypad {
        pub fn new(rows: super::KeypadRows, columns: super::KeypadColumns) -> Self {
            Self { rows, columns }
        }

        fn read_column(&self, delay: &mut Delay) -> u16 {
            let mut res = 0;
            
            delay.delay_ms(1u16);
            if self.rows.0.is_low().unwrap() {
                res |= 1 << 0;
            }
            if self.rows.1.is_low().unwrap() {
                res |= 1 << 1;
            }
            if self.rows.2.is_low().unwrap() {
                res |= 1 << 2;
            }
            if self.rows.3.is_low().unwrap() {
                res |= 1 << 3;
            }
            
            res
        }

        pub fn read(&mut self, delay: &mut Delay) -> u16 {
            let mut res = 0;
            
            self.columns.0.set_low().unwrap();
            res |= self.read_column(delay) << 0;
            self.columns.0.set_high().unwrap();
            
            self.columns.1.set_low().unwrap();
            res |= self.read_column(delay) << 4;
            self.columns.1.set_high().unwrap();
            
            self.columns.2.set_low().unwrap();
            res |= self.read_column(delay) << 8;
            self.columns.2.set_high().unwrap();
            
            res
        }
    }
}