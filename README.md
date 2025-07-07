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