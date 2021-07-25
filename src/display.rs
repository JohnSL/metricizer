pub mod display {
    use core::{fmt::Display, u8};

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
        show_function: u8,
        show_mode: u8,
        control: DisplayControl,
    }

    struct DisplayControl {
        control: u8,
    }

    impl DisplayControl {
        pub fn new() -> DisplayControl {
            DisplayControl { control: 0 }
        }

        pub fn set(&mut self, value: ControlOptions) -> &mut Self {
            self.control |= value as u8;
            self
        }

        pub fn clear(&mut self, value: ControlOptions) -> &mut Self {
            self.control &= !(value as u8);
            self
        }

        pub fn value(&mut self) -> u8 {
            self.control
        }
    }

    impl<I> Lcd<I>
    where
        I: i2c::Write
        {
        pub fn new(i2c: I) -> Self {
            let lcd = Lcd {
                i2c: i2c,
                show_function: LCD_4BITMODE | LCD_2LINE | LCD_5X8_DOTS,
                show_mode: LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT,
                control: DisplayControl::new()
            };
            lcd
        }

        //
        // Initialize the display for the first time after power up
        //
        pub fn init(&mut self, delay: &mut Delay)  -> Result<(), <I as i2c::Write>::Error> {
            delay.delay_ms(80_u16); // Need to wait at least 40ms before sending commands

            // Send the initial command sequence according to the HD44780 datasheet
            self.command(LCD_FUNCTIONSET | self.show_function, delay)?;
            delay.delay_ms(5_u16);

            self.command(LCD_FUNCTIONSET | self.show_function, delay)?;
            delay.delay_ms(5_u16);

            self.command(LCD_FUNCTIONSET | self.show_function, delay)?;

            // Turn on the display wit no cursor or blinking
            self.control.set(ControlOptions::DisplayOn);
            self.update_display(delay)?;

            self.clear(delay)?;

            self.command(LCD_ENTRYMODESET | self.show_mode, delay)?;

            // Initialize the backlight
            self.set_reg(REG_MODE1, 0)?;

            // Set the LEDs controllable by both PWM and GRPPWM registers
            self.set_reg(REG_OUTPUT, 0xFF)?;
            self.set_reg(REG_MODE2, 0x20)?;

            self.set_rgb(255, 255, 255)
        }

        // Clear the display
        pub fn clear(&mut self, delay: &mut Delay) -> Result<(), <I as i2c::Write>::Error> {
            let result = self.command(LCD_CLEARDISPLAY, delay);
            delay.delay_ms(2_u32);
            result
        }

        // Set the position of the cursor
        pub fn cursor_position(&mut self, x: u8, y: u8, delay: &mut Delay) -> Result<(), <I as i2c::Write>::Error> {
            let col = if y == 0_u8 { x | 0x80 } else { x | 0xC0 };
            self.send_two(0x80, col, delay)
        }

        // Turns on the cursor, which is a non-blinking _
        pub fn cursor_on(&mut self, delay: &mut Delay) -> Result<(), <I as i2c::Write>::Error> {
            self.control.set(ControlOptions::CursorOn);
            self.update_display(delay)
        }

        pub fn blink_on(&mut self, delay: &mut Delay) -> Result<(), <I as i2c::Write>::Error> {
            self.control.set(ControlOptions::BlinkOn);
            self.update_display(delay)
        }

        pub fn send_char(&mut self, char: char, delay: &mut Delay) -> Result<(), <I as i2c::Write>::Error> {
            self.send_two(0x40, char as u8, delay)
        }

        // Send a command to the LCD display
        fn command(&mut self, value: u8, delay: &mut Delay) -> Result<(), <I as i2c::Write>::Error> {
            self.send_two(0x80, value, delay)
        }

        fn send_two(&mut self, byte1: u8, byte2: u8, delay: &mut Delay) -> Result<(), <I as i2c::Write>::Error> {
            let result = self.i2c.write(LCD_ADDRESS, &[byte1, byte2]);
            delay.delay_ms(5_u16);
            result
        }

        fn set_reg(&mut self, addr: u8, data: u8) -> Result<(), <I as i2c::Write>::Error> {
            self.i2c.write(RGB_ADDRESS, &[addr, data])
        }

        // Set the color of the backlight
        fn set_rgb(&mut self, r: u8, g: u8, b: u8) -> Result<(), <I as i2c::Write>::Error> {
            self.set_reg(REG_RED, r)?;
            self.set_reg(REG_GREEN, g)?;
            self.set_reg(REG_BLUE, b)
        }

        fn update_display(&mut self, delay: &mut Delay) -> Result<(), <I as i2c::Write>::Error> {
            let value = self.control.value();
            self.command(LCD_DISPLAYCONTROL | value, delay)
        }
    }

    // Device I2c addresses
    const LCD_ADDRESS: u8 = 0x7c >> 1;
    const RGB_ADDRESS: u8 = 0xc0 >> 1;

    // Commands
    const LCD_CLEARDISPLAY: u8 = 0x01;
    const LCD_ENTRYMODESET: u8 = 0x04;
    const LCD_DISPLAYCONTROL: u8 = 0x08;
    const LCD_FUNCTIONSET: u8 = 0x20;

    // Flags for display on/off control
    #[repr(u8)]
    enum ControlOptions {
        DisplayOn = 0x04,
        Off = 0x0,
        CursorOn = 0x02,
        BlinkOn = 0x01,
    }

    const LCD_8BITMODE: u8 = 0x10;
    const LCD_4BITMODE: u8 = 0x00;
    const LCD_2LINE: u8 = 0x08;
    const LCD_1LINE: u8 = 0x00;
    const LCD_5X8_DOTS: u8 = 0x00;
        
    // Display entry mode
    // const LCD_ENTRYRIGHT: u8 = 0x00;
    const LCD_ENTRYLEFT: u8 = 0x02;
    // const LCD_ENTRYSHIFTINCREMENT: u8 = 0x01;
    const LCD_ENTRYSHIFTDECREMENT: u8 = 0x00;
    
    
    // Colors
    // const WHITE: u8         = 0;
    // const RED: u8           = 1;
    // const GREEN: u8         = 2;
    // const BLUE: u8          = 3;

    const REG_RED: u8       = 0x04;        // pwm2
    const REG_GREEN: u8     = 0x03;        // pwm1
    const REG_BLUE: u8      = 0x02;        // pwm0
    
    const REG_MODE1: u8     = 0x00;
    const REG_MODE2: u8     = 0x01;
    const REG_OUTPUT: u8    = 0x08;
    }
