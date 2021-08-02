pub type Rows<R0, R1, R2, R3> = (R0, R1, R2, R3);

pub type Columns<C0, C1, C2> = (C0, C1, C2);

pub mod keypad {
    use embedded_hal::digital::v2::{InputPin, OutputPin};
    use stm32f1xx_hal::{delay::Delay, prelude::*};

    pub struct MyKeypad<
        R0: InputPin,
        R1: InputPin,
        R2: InputPin,
        R3: InputPin,
        C0: OutputPin,
        C1: OutputPin,
        C2: OutputPin,
    > {
        rows: super::Rows<R0, R1, R2, R3>,
        columns: super::Columns<C0, C1, C2>,
    }

    impl<
            R0: InputPin,
            R1: InputPin,
            R2: InputPin,
            R3: InputPin,
            C0: OutputPin,
            C1: OutputPin,
            C2: OutputPin,
        > MyKeypad<R0, R1, R2, R3, C0, C1, C2>
    {
        pub fn new(rows: super::Rows<R0, R1, R2, R3>, columns: super::Columns<C0, C1, C2>) -> Self {
            Self { rows, columns }
        }

        /// Reads a character from the keypad. This method returns even if no keys are pressed.
        ///
        /// Returns ' ' if no keys are pressed.
        pub fn read_char(&mut self, delay: &mut Delay) -> char {
            let raw = self.read(delay);
            if raw != 0 {
                self.get_char(raw)
            } else {
                ' '
            }
        }

        // Performs a "raw" read of the keypad and returns a bit set for each key down. Note,
        // this doesn't mean this code supports multiple key presses.
        fn read(&mut self, delay: &mut Delay) -> u16 {
            let mut res = 0;

            self.columns.0.set_low().unwrap_or_default();
            res |= self.read_column(delay) << 0;
            self.columns.0.set_high().unwrap_or_default();

            self.columns.1.set_low().unwrap_or_default();
            res |= self.read_column(delay) << 4;
            self.columns.1.set_high().unwrap_or_default();

            self.columns.2.set_low().unwrap_or_default();
            res |= self.read_column(delay) << 8;
            self.columns.2.set_high().unwrap_or_default();

            res
        }

        // Converts the raw value from the read() method into a character that corresponds to the
        // label on each key
        fn get_char(&self, raw_value: u16) -> char {
            let value = self.convert(raw_value);
            match value {
                -1 => '*',
                -2 => '#',
                _ => char::from_digit(value as u32, 10).unwrap(),
            }
        }

        fn read_column(&self, delay: &mut Delay) -> u16 {
            let mut res = 0;

            delay.delay_ms(1u16);
            if self.rows.0.is_low().unwrap_or_default() {
                res |= 1 << 0;
            }
            if self.rows.1.is_low().unwrap_or_default() {
                res |= 1 << 1;
            }
            if self.rows.2.is_low().unwrap_or_default() {
                res |= 1 << 2;
            }
            if self.rows.3.is_low().unwrap_or_default() {
                res |= 1 << 3;
            }

            res
        }

        // Converts the raw value (2^N) from the read() method into a keypad digit. This will be
        //      0..9    digits
        //      -1      *
        //      -2      #
        pub fn convert(&self, value: u16) -> i16 {
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
                _ => -10,
            }
        }
    }
}