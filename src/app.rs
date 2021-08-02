use core::fmt::Write;
use embedded_hal::blocking::i2c;
use heapless::String;
use lcd_1602_i2c::Lcd;

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
        let mut line1: String<16> = String::new();
        let mut line2: String<16> = String::new();

        line1.push_str(&self.entered).unwrap();

        let cursor = if self.entered.len() > 0 {
            self.lcd.cursor_on()?;
            self.entered.len() as u8
        } else {
            self.lcd.cursor_off()?;
            0
        };

        self.add_input(&mut line1, &self.entered, "\"");
        self.add_input(&mut line2, &self.entered, "mm");

        let input: f32 = self.entered.parse().unwrap_or(-1.);
        if input > 0. {
            // Inches to mm
            self.add_number(&mut line1, input * 25.4, "mm");

            // mm to inches
            self.add_number(&mut line2, input / 25.4, "\"");
        } else {
            write!(&mut line1, "{:16}", " ").unwrap_or_default();
            write!(&mut line2, "{:16}", " ").unwrap_or_default();
        }

        // Draw the inches to mm line
        self.lcd.cursor_position(0, 0)?;
        self.lcd.print(&line1)?;

        // Draw the mm to inches line
        self.lcd.cursor_position(0, 1)?;
        self.lcd.print(&line2)?;

        self.lcd.cursor_position(cursor, 0)
    }

    fn add_input(&self, line: &mut String<16>, input: &String<16>, suffix: &str) {
        line.clear();

        if input.len() == 0 {
            write!(line, "{:8}", "0").unwrap_or_default();
        } else {
            let mut number: String<8> = String::new();
            number.push_str(&input).unwrap_or_default();
            number.push_str(&suffix).unwrap_or_default();
            write!(line, "{:7}", number).unwrap_or_default();
        }
    }

    fn add_number(&self, line: &mut String<16>, number: f32, suffix: &str) {
        let _ = write!(line, "{:7.4}{}", number, suffix);
    }
}
