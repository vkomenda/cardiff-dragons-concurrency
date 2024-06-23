//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

// use core::hint::spin_loop;
// use core::sync::atomic::{AtomicBool, Ordering};
use bsp::entry;
use core::ptr;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::OutputPin;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::{
    hal::{
        clocks::{init_clocks_and_plls, Clock, ClocksManager},
        multicore::{Multicore, Stack},
        sio::Sio,
        watchdog::Watchdog,
    },
    pac,
};

// Atomic flag to indicate to core1 that core0 is ready
// static CORE0_READY: AtomicBool = AtomicBool::new(false);

// Allows the clock settings to be read from core1 after initialisation on core0
static mut CLOCKS_MANAGER: *mut ClocksManager = ptr::null_mut();

// Constants for LED blinking
const BLINK_DELAY1: u32 = 500; // 0.5 seconds
const BLINK_DELAY2: u32 = 250; // 0.25 seconds

static mut CORE1_STACK: Stack<4096> = Stack::new();

// The protocol for launching core1 as described in the RP2040 datasheet.
// fn launch_core1(sio: &mut Sio) {
//     // sp is initial stack pointer (SP)
//     // entry is the initial program counter (PC) (don't forget to set the thumb bit!)
//     let cmd_sequence: [u32; 6] = [0, 0, 1, vector_table, sp, main_core1 as u32];

//     let mut seq = 0;
//     while seq < cmd_sequence.len() {
//         let cmd = cmd_sequence[seq];
//         // always drain the READ FIFO (from core 1) before sending a 0
//         if cmd == 0 {
//             // discard data from read FIFO until empty
//             sio.fifo.drain();
//             // execute a SEV as core 1 may be waiting for FIFO space
//             cortex_m::asm::sev();
//         }
//         // write 32 bit value to write FIFO
//         sio.fifo.write_blocking(cmd);
//         // read 32 bit value from read FIFO once available
//         let response = sio.fifo.read_blocking();
//         // move to next state on correct response (echo-d value) otherwise start over
//         seq = if cmd == response { seq + 1 } else { 0 };
//     }
// }

// Core 1 entry function
fn main_core1() {
    // Wait for core0 to indicate readiness
    // while !CORE0_READY.load(Ordering::Acquire) {
    //     // Compiler hint to indicate a busy-wait loop
    //     spin_loop();
    // }

    // Set up core 1 peripherals
    let mut pac = unsafe { pac::Peripherals::steal() };
    let core = unsafe { pac::CorePeripherals::steal() };
    let sio = Sio::new(pac.SIO);
    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let clocks = unsafe { CLOCKS_MANAGER.as_mut().unwrap() };

    info!("core1 freq: {}", clocks.system_clock.freq().to_Hz());

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // Configure GPIO 16 as output
    let mut led_pin = pins.gpio16.into_push_pull_output();

    loop {
        info!("ON");
        led_pin.set_high().unwrap();
        delay.delay_ms(BLINK_DELAY2);
        info!("OFF");
        led_pin.set_low().unwrap();
        delay.delay_ms(BLINK_DELAY2);
    }
}

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    // External high-speed crystal on the pico board is 12Mhz
    let mut clocks = init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    unsafe {
        CLOCKS_MANAGER = &mut clocks;
    }

    info!("core0 freq: {}", clocks.system_clock.freq().to_Hz());

    let mut sio = Sio::new(pac.SIO);
    let mut multicore = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let cores = multicore.cores();
    let core1 = &mut cores[1];
    if let Err(e) = core1.spawn(unsafe { &mut CORE1_STACK.mem }, main_core1) {
        error!("Cannot start core1: {}", e);
    }

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // Indicate core1 can use the clocks
    // CORE0_READY.store(true, Ordering::Release);

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // This is the correct pin on the Raspberry Pico board. On other boards, even if they have an
    // on-board LED, it might need to be changed.
    //
    // Notably, on the Pico W, the LED is not connected to any of the RP2040 GPIOs but to the cyw43 module instead.
    // One way to do that is by using [embassy](https://github.com/embassy-rs/embassy/blob/main/examples/rp/src/bin/wifi_blinky.rs)
    //
    // If you have a Pico W and want to toggle a LED with a simple GPIO output pin, you can connect an external
    // LED to one of the GPIO pins, and reference that pin here. Don't forget adding an appropriate resistor
    // in series with the LED.
    let mut led_pin = pins.led.into_push_pull_output();

    loop {
        info!("on!");
        led_pin.set_high().unwrap();
        delay.delay_ms(BLINK_DELAY1);
        info!("off!");
        led_pin.set_low().unwrap();
        delay.delay_ms(BLINK_DELAY1);
    }
}

// End of file
