#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_semihosting;

use cortex_m_semihosting::{hprintln};

use rtfm::app;

use nrf52840_hal::{gpio, prelude::*, uarte, };

use nrf52840_pac as pac;

// const PWM_RISING_EDGE: u16 = 0x0000;
const PWM_FALLING_EDGE: u16 = 0x8000;

// 16MHz -> 62.5 ns
// 62.5 ns * 20 -> 1.25 us
const NEOPIXEL_COUNTER_TOP: u16 = 20;
// 62.5 ns * 6 -> 0.375 us
const NEOPIXEL_TIME_0_HIGH: u16 = 6 | PWM_FALLING_EDGE;
// 62.5ns * 13 -> 0.8125us
const NEOPIXEL_TIME_1_HIGH: u16 = 13 | PWM_FALLING_EDGE;

#[app(device = nrf52840_pac, peripherals = true)]
const APP: () = {
    struct Resources {
        pwm: pac::PWM0,
        timer: pac::TIMER0,
        uart: uarte::Uarte<pac::UARTE0>,
        #[init(0)]
        counter: u8,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let p0 = cx.device.P0.split();
        // Configure to use external clocks, and start them
        // The Adafruit Circuit Playground Bluefruit do not have an external 32.768 kHz source
        let _clocks = cx.device
            .CLOCK
            .constrain()
            .enable_ext_hfosc()
            .set_lfclk_src_synth()
            .start_lfclk();

        let uarte0 = cx.device.UARTE0.constrain(
            uarte::Pins {
                txd: p0.p0_14.into_push_pull_output(gpio::Level::High).degrade(),
                rxd: p0.p0_30.into_floating_input().degrade(),
                cts: None,
                rts: None,
            },
            uarte::Parity::EXCLUDED,
            uarte::Baudrate::BAUD115200,
        );

        let timer = cx.device.TIMER0;
        timer.mode.write(|w| w.mode().timer());
        timer.bitmode.write(|w| w.bitmode()._32bit());
        timer.shorts.write(|w| w.compare0_clear().enabled().compare0_stop().disabled());
        timer.prescaler.write(|w| unsafe { w.prescaler().bits(4) });
        timer.cc[0].write(|w| unsafe { w.bits(50_000) });
        timer.intenset.write(|w| w.compare0().set());
        timer.tasks_clear.write(|w| w.tasks_clear().set_bit());
        timer.tasks_start.write(|w| w.tasks_start().set_bit());

        let pwm = cx.device.PWM0;

        // Only count up
        pwm.mode.write(|w| w.updown().up());
        // Set a PWM period
        pwm.countertop.write(|w| unsafe { w.countertop().bits(NEOPIXEL_COUNTER_TOP) });
        // Divide by one (1) prescaler, gives us a frequency of 16MHz
        pwm.prescaler.write(|w| w.prescaler().div_1());
        pwm.decoder.write(|w| w.load().common().mode().clear_bit());
        // Neopixel is connected t0 P0.13
        pwm.psel.out[0].write(|w| unsafe { w.pin().bits(13).port().bit(false).connect().connected() });
        pwm.loop_.write(|w| w.cnt().disabled());
        // Enable the PWM instance
        pwm.enable.write(|w| w.enable().set_bit());

        init::LateResources {
            pwm,
            timer,
            uart: uarte0,
        }
    }

    #[task(binds = TIMER0, resources = [pwm, counter, timer])]
    fn timer(cx: timer::Context) {
        let pwm = cx.resources.pwm;
        let counter = cx.resources.counter;
        cx.resources.timer.events_compare[0].reset();
        let mut colours = [0u8; 30];
        let mut pwm_words = [0x8000u16; 30 * 8 + 2];

        *counter = (*counter + 1) % 10;
        let index = *counter as usize;

        // continous operation
        pwm.seq0.refresh.write(|w| w.cnt().continuous());
        // No end delay
        pwm.seq0.enddelay.write(|w| unsafe { w.cnt().bits(0) } );
        // data counter
        pwm.seq0.cnt.write(|w| unsafe { w.cnt().bits(pwm_words.len() as u16) } );
        // point to data
        let pwm_ptr = &mut pwm_words as *mut _ as u32;
        pwm.seq0.ptr.write(|w| unsafe { w.ptr().bits(pwm_ptr) } );
        // Enable sequence end event
        pwm.intenset.write(|w| w.seqend0().set() );

        for (n, colour) in colours.chunks_exact_mut(3).enumerate() {
            if n == index {
                colour[0] = 0; // green
                colour[1] = 0; // red
                colour[2] = 1; // blue
            }
            else if n == ((index + 1) % 10) {
                colour[0] = 0; // green
                colour[1] = 0; // red
                colour[2] = 3; // blue
            }
            else if n == ((index + 2) % 10) {
                colour[0] = 0; // green
                colour[1] = 0; // red
                colour[2] = 4; // blue
            }
            else if n == ((index + 3) % 10) {
                colour[0] = 0; // green
                colour[1] = 0; // red
                colour[2] = 32; // blue
            }
            else {
                colour[0] = 0; // green
                colour[1] = 0; // red
                colour[2] = 0; // blue
            }
        }
        for (words, byte) in pwm_words.chunks_exact_mut(8).zip(colours.iter()) {
            let mut b = *byte;
            for n in 0..8 {
                let word = if b & 0x80 == 0x80 { NEOPIXEL_TIME_1_HIGH } else { NEOPIXEL_TIME_0_HIGH };
                words[n] = word;
                b <<= 1;
            }
        }
        pwm.events_seqend[0].write(|w| w.events_seqend().clear_bit());
        pwm.tasks_seqstart[0].write(|w| w.tasks_seqstart().set_bit());
        while pwm.events_seqend[0].read().events_seqend().bit_is_clear() {
        }
        pwm.events_seqend[0].write(|w| w.events_seqend().clear_bit());
    }


    #[idle(resources = [uart])]
    fn idle(cx: idle::Context) -> ! {
        let uarte = cx.resources.uart;

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
