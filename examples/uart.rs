#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_semihosting;

use cortex_m_semihosting::{hprintln};

use rtfm::app;

use nrf52840_hal::{gpio, prelude::*, uarte};

use nrf52840_pac as pac;

#[app(device = nrf52840_pac)]
const APP: () = {
    static mut UARTE: uarte::Uarte<pac::UARTE0> = ();

    #[init]
    fn init() {
        let p0 = device.P0.split();
        // Configure to use external clocks, and start them
        // The Adafruit Circuit Playground Bluefruit do not have an external 32.768 kHz source
        let _clocks = device
            .CLOCK
            .constrain()
            .enable_ext_hfosc()
            .set_lfclk_src_synth()
            .start_lfclk();

        let uarte0 = device.UARTE0.constrain(
            uarte::Pins {
                txd: p0.p0_14.into_push_pull_output(gpio::Level::High).degrade(),
                rxd: p0.p0_30.into_floating_input().degrade(),
                cts: None,
                rts: None,
            },
            uarte::Parity::EXCLUDED,
            uarte::Baudrate::BAUD115200,
        );

        hprintln!(" ~ Init done ~ ").unwrap();

        UARTE = uarte0;
    }

    #[idle(resources = [UARTE])]
    fn idle() -> ! {
        let uarte = resources.UARTE;
        hprintln!(" ~ Idle ~ ").unwrap();
        let mut buffer = [0u8; 128];
        let message = "Hello, World\r\n";
        let bytes = message.as_bytes();
        let length = bytes.len();
        buffer[..length].copy_from_slice(&bytes);

        loop {
            uarte.write(&buffer[..length]).unwrap();
        }
    }
};
