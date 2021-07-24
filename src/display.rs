pub mod display {
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
