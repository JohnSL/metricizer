use lcd_1602_i2c::Lcd;
use embedded_hal::blocking::i2c;
use heapless::String;
use core::{fmt::Write};

pub struct MainApp<I>
where
    I: i2c::Write,
{
    lcd: Lcd<I>,
    entered: String<16>,
    dot: bool,
}

impl<I> MainApp<I>
where
    I: i2c::Write,
{
    pub fn new(lcd: Lcd<I>) -> Result<MainApp<I>, <I as i2c::Write>::Error>
    where
        I: i2c::Write,
    {
        let mut app = MainApp {
            lcd: lcd,
            entered: String::new(),
            dot: false,
        };
        app.init()?;
        Ok(app)
    }

    fn init(&mut self) -> Result<(), <I as i2c::Write>::Error> {
        self.lcd.set_rgb(255, 255, 255)?;
        self.lcd.cursor_on()
    }

    pub fn clear(&mut self) -> Result<(), <I as i2c::Write>::Error> {
        self.key('*')
    }

    pub fn key(&mut self, key: char) -> Result<(), <I as i2c::Write>::Error> {
        match key {
            '*' => {
                self.entered.clear();
                self.dot = false;
            }
            '#' => {
                if !self.dot {
                    self.entered.push('.').unwrap();
                    self.dot = true;
                }
            }
            _ => {
                // Allow only one zero to be entered. But also replace a loan zero with a digit
                // if that is the first key after a zero.
                if key != '0' && self.entered == "0" {
                    self.entered.clear();
                }
                
                if key != '0' || self.entered != "0" {
                    self.entered.push(key).unwrap();
                }
            }
        }

        self.update()
    }

    pub fn update(&mut self) -> Result<(), <I as i2c::Write>::Error> {
        // let f: f32 = self.entered.parse().unwrap();

        let mut line1: String<16> = String::new();
        let mut line2: String<16> = String::new();
        let blank: bool;

        line1.push_str(&self.entered).unwrap();

        let cursor = if self.entered.len() > 0 {
            blank = false;
            self.lcd.cursor_on()?;
            self.entered.len() as u8
        } else {
            blank = true;
            self.lcd.cursor_off()?;
            line1.push('0').unwrap();
            0
        };

        line2.push_str(&line1).unwrap();
        if !blank {
            line1.push('"').unwrap();
            line2.push_str("mm").unwrap();
        }

        pad(&mut line1, 8);
        pad(&mut line2, 8);

        let mut converted: String<8> = String::new();

        let input: f32 = self.entered.parse().unwrap_or(-1.);
        if input > 0. {
            // Inches to mm
            let _ = write!(converted, "{:7.4}", input * 25.4);
            line1.push_str(&converted).unwrap_or(());
            line1.push_str("m").unwrap_or(());

            // mm to inches
            converted.clear();
            let _ = write!(converted, "{:7.4}", input / 25.4);
            line2.push_str(&converted).unwrap_or(());
            line2.push('"').unwrap_or(());
        }

        pad_line(&mut line1);
        pad_line(&mut line2);

        // Draw the inches to mm line
        self.lcd.cursor_position(0, 0)?;
        self.lcd.print(&line1)?;

        // Draw the mm to inches line
        self.lcd.cursor_position(0, 1)?;
        self.lcd.print(&line2)?;

        self.lcd.cursor_position(cursor, 0)
    }
}

// Adds spaces to the end of the line, which effectively clears the "right" side of the line.
fn pad(line: &mut String<16>, len: usize) {
    for _ in 0..len - line.len() {
        line.push(' ').unwrap();
    }
}

// Adds spaces to the end of the line, which effectively clears the "right" side of the line.
fn pad_line(line: &mut String<16>) {
    for _ in 0..line.capacity() - line.len() {
        line.push(' ').unwrap();
    }
}
