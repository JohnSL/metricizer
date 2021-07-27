use stm32f1xx_hal as hal;
use hal::gpio::{Input, OpenDrain, Output, PullUp};
use hal::gpio::gpioa::{PA7, PA8, PA9};
use hal::gpio::gpiob::{PB5, PB6, PB15};
use hal::gpio::gpioc::{PC7};

// Pin      Keypad              Keypad  Pin
// ----     ------              ------  ---
// PA7      R2                  R1      PB15
// PB6      R3                  R2      PA7
// PC7      C3                  R3      PB6
// PA9      R4                  R4      PA9
// PA8      C1                  C1      PA8
// PB15     R1                  C2      PB5
// PB5      C2                  C3      PC7
pub type KeypadRows = (
    PB15<Input<PullUp>>,
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

        // Converts the raw value from the read() method into a keypad digit. This will be
        //      0..9    digits
        //      -1      *
        //      -2      #
        pub fn convert(&mut self, value: u16) -> i16 {
            match value {
                1 => 1,
                2 => 4,
                4 => 7,
                8 => -1,
                16 => 2,
                32 => 5,
                64 => 8,
                128 => 0,
                256 => 3,
                512 => 6,
                1024 => 9,
                2048 => -2,
                _ => -10
            }
        }
    }
}