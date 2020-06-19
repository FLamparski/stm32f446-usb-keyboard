#![no_std]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate panic_semihosting;
extern crate stm32f4xx_hal as hal;

use cortex_m_rt::{entry, exception, ExceptionFrame};
use crate::hal::{
    prelude::*,
    stm32,
    otg_fs::{USB, UsbBus},
};
use usb_device::prelude::*;
use usbd_hid_device;


pub mod keyboard;
use keyboard::*;

// USB endpoint memory
static mut EP_MEMORY: [u32; 1024] = [0; 1024];

#[entry]
fn main() -> ! {
    if let Some(dp) = stm32::Peripherals::take() {
        let rcc = dp.RCC.constrain();
        let _clocks = rcc.cfgr
            .use_hse(8.mhz())
            .sysclk(72.mhz())
            .pclk1(36.mhz())
            .pclk2(72.mhz())
            .require_pll48clk()
            .freeze();

        let gpiob = dp.GPIOB.split();
        let mut led_grn = gpiob.pb0.into_push_pull_output();
        let mut led_blu = gpiob.pb7.into_push_pull_output();
        let mut led_red = gpiob.pb14.into_push_pull_output();

        let gpioc = dp.GPIOC.split();
        let btn = gpioc.pc13.into_pull_down_input();
        let mut btn_was_pressed = btn.is_high().unwrap();

        let gpioa = dp.GPIOA.split();

        let usb = USB {
            usb_global: dp.OTG_FS_GLOBAL,
            usb_device: dp.OTG_FS_DEVICE,
            usb_pwrclk: dp.OTG_FS_PWRCLK,
            pin_dm: gpioa.pa11.into_alternate_af10(),
            pin_dp: gpioa.pa12.into_alternate_af10(),
        };

        let usb_bus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });
        // let mut serial = usbd_serial::SerialPort::new(&usb_bus);

        let mut hid = usbd_hid_device::Hid::<KeyboardReport, _>::new(&usb_bus, 10);
        let text = "Hello!\r\n";
        let mut kbd = Keyboard::new();

        let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Fake company")
            .product("Keyboard")
            .serial_number("TEST")
            .device_class(usbd_hid_device::USB_CLASS_HID)
            .build();

        let mut state = State::idle();

        loop {
            led_red.set_low().unwrap();
            let btn_state = btn.is_high().unwrap();
            let was_pressed = btn_state && !btn_was_pressed;
            btn_was_pressed = btn_state;

            if was_pressed && matches!(state, State::Idle) {
                state = state.set_keys(text.as_bytes()).unwrap();
            }

            led_grn.set_low().unwrap();
            if matches!(state, State::Keying(_) | State::Releasing(_)) {
                led_grn.set_high().unwrap();
            }

            if usb_dev.poll(&mut [&mut hid]) {
                match state {
                    State::Keying(&[]) | State::Releasing(_) => kbd.release_all(),
                    State::Keying(text) => kbd.press(text[0]),
                    _ => Ok(())
                }.unwrap();

                match hid.send_report(&kbd.get_report()) {
                    Ok(_) => state = state.next(),
                    Err(UsbError::WouldBlock) => led_red.set_high().unwrap(),
                    Err(e) => panic!("{:?}", e),
                }
            }
        }
    }

    loop {}
}

enum State<'a> {
    Idle,
    Keying(&'a[u8]),
    Releasing(&'a[u8]),
}

impl <'a> State<'a> {
    fn idle() -> Self {
        Self::Idle
    }

    fn next(&self) -> Self {
        match self {
            Self::Idle => Self::Idle,
            Self::Keying(&[]) => Self::Idle,
            Self::Keying(text) => Self::Releasing(text),
            Self::Releasing(text) => Self::Keying(&text[1..text.len()]),
        }
    }

    fn set_keys(&self, keys: &'a[u8]) -> Option<Self> {
        match self {
            Self::Idle => Some(Self::Keying(keys)),
            _ => None,
        }
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
