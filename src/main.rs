// src/main.rs

#![no_std]
#![no_main]

use cortex_m_rt::entry; // The runtime

use stm32f1xx_hal as hal;
use hal::{delay::Delay, pac, prelude::*}; // STM32F1 specific functions+

#[allow(unused_imports)]
use panic_halt; // When a panic occurs, stop the microcontroller

use embedded_hal::digital::v2::{InputPin, OutputPin};
use hal::gpio::{Input, OpenDrain, Output, PullUp};
use hal::gpio::gpioa::{PA7, PA8, PA9};
use hal::gpio::gpiob::{PB5, PB6, PB10};
use hal::gpio::gpioc::{PC7};

// keypad_struct!{
//     struct MyKeypad {
//         rows: (
//             PB6<Input<PullUp>>,
//             PB5<Input<PullUp>>,
//             PB10<Input<PullUp>>,
//             PA9<Input<PullUp>>,
//         ),
//         columns: (
//             PC7<Output<OpenDrain>>,
//             PA7<Output<OpenDrain>>,
//             PA8<Output<OpenDrain>>,
//         ),
//     }
// }

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    let mut flash = dp.FLASH.constrain();

    let clocks = rcc.cfgr.sysclk(8.mhz()).freeze(&mut flash.acr);

    let mut delay = Delay::new(cp.SYST, clocks);

    let mut keypad = MyKeypad::new((
            gpiob.pb6.into_pull_up_input(&mut gpiob.crl),
            gpiob.pb5.into_pull_up_input(&mut gpiob.crl),
            gpiob.pb10.into_pull_up_input(&mut gpiob.crh),
            gpioa.pa9.into_pull_up_input(&mut gpioa.crh)
        ),(
            gpioc.pc7.into_open_drain_output(&mut gpioc.crl),
            gpioa.pa7.into_open_drain_output(&mut gpioa.crl),
            gpioa.pa8.into_open_drain_output(&mut gpioa.crh)
        )
    );

    loop {
        let f: f32 = 1.2;
        let v = keypad.read();
        delay.delay_ms(f as u16);
    }
}

// Pin      Keypad              Keypad  Pin
// ----     ------              ------  ---
// PA7      C2                  C1      PC7
// PB6      R1                  C2      PA7
// PC7      C1                  C3      PA8
// PA9      R4                  R1      PB6
// PA8      C3                  R2      PB5
// PB10     R3                  R3      PB10
// PB5      R2                  R4      PA9

pub type KeypadRows = (
	PB6<Input<PullUp>>,
	PB5<Input<PullUp>>,
	PB10<Input<PullUp>>,
	PA9<Input<PullUp>>,
);

pub type KeypadColumns = (
	PC7<Output<OpenDrain>>,
	PA7<Output<OpenDrain>>,
	PA8<Output<OpenDrain>>,
);

pub struct MyKeypad {
	rows: KeypadRows,
	columns: KeypadColumns,
}

impl MyKeypad {
	pub fn new(rows: KeypadRows, columns: KeypadColumns) -> Self {
		Self { rows, columns }
	}

	fn read_column(&self) -> u16 {
		let mut res = 0;
		
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
	
	pub fn read(&mut self) -> u16 {
		let mut res = 0;
		
		self.columns.0.set_low().unwrap();
		// Add delay here?
		res |= self.read_column() << 0;
		self.columns.0.set_high().unwrap();
		
		self.columns.1.set_low().unwrap();
		// Add delay here?
		res |= self.read_column() << 4;
		self.columns.1.set_high().unwrap();
		
		self.columns.2.set_low().unwrap();
		// Add delay here?
		res |= self.read_column() << 8;
		self.columns.2.set_high().unwrap();
		
		res
	}
}
