# stm32-hrtim

This crate implements a driver for the HRTIM peripheral found in select devices from F3, G4 and H7 series of STM32 micro controllers. The HRTIM peripheral is a high resolution timer that allows precise control even at higher frequencies.

The HRTIM is primarily intended to drive power conversion systems such as switch mode power
supplies or lighting systems, but can be of general purpose usage, whenever a very fine
timing resolution is expected. It is very flexible allowing complicated waveforms and support connections to various other peripherals like DACs, ADCs, comparators, other timers, DMA etc.

### Status

🚧 Work in progress

This crate is being developed and lots of things are still subject to change. It should still be considered quite experimental. Use with cation.

|   Device  |     Status     |
|-----------|----------------|
| STM32F3x4 |      TODO      |
| STM32G474 | Mostly working |
| STM32G484 | Mostly working |
| stm32h742 |      TODO      |
| stm32h743 |      TODO      |
| stm32h745 |      TODO      |
| stm32h747 |      TODO      |
| stm32h750 |      TODO      |
| stm32h753 |      TODO      |
| stm32h755 |      TODO      |
| stm32h757 |      TODO      |

### Usage
This driver is intended for use through a device hal library. See [stm32g4xx-hal](https://github.com/stm32-rs/stm32g4xx-hal/) as a reference.

```rust
//        .               .               .               .
//        .  30%          .               .               .
//         ----           .               .----           .
//out1    |    |          .               |    |          .
//        |    |          .               |    |          .
// --------    ----------------------------    --------------------
//        .               .----           .               .----
//out2    .               |    |          .               |    |
//        .               |    |          .               |    |
// ------------------------    ----------------------------    ----
//        .               .               .               .
//        .               .               .               .
let (hr_control, ..) = dp.HRTIM_COMMON.hr_control(&mut rcc).wait_for_calibration();
let mut hr_control = hr_control.constrain();

let HrParts {
    mut timer,
    mut cr1,
    mut out1,
    mut out2,
    ..
} = dp
    .HRTIM_TIMA
    .pwm_advanced(pin_a, pin_b)
    .prescaler(prescaler)
    .period(0xFFFF)
    .push_pull_mode(true) // Set push pull mode, out1 and out2 are
    // alternated every period with one being
    // inactive and the other getting to output its wave form
    // as normal
    .finalize(&mut hr_control);

out1.enable_rst_event(&cr1); // Set low on compare match with cr1
out2.enable_rst_event(&cr1);

out1.enable_set_event(&timer); // Set high at new period
out2.enable_set_event(&timer);

out1.enable();
out2.enable();

timer.start(&mut hr_control.control);
```