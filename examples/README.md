# SSD1677 Examples

This folder contains examples showing how to use the SSD1677 crate on an STM32U575 with
[Embassy](https://embassy.dev/).  

The following examples are provided:

- [Embedded Graphics Example](#embedded-graphics-example)
- [Clock Example](#clock-example)

For all examples the STM32 is connected to the display as follows:

![Image showing the connections between the STM and the Display](./images/ssd1677_connection_diagram.jpg)

## Embedded Graphics Example

[This example](./embedded-graphics-example/) shows a variant of the standard [Embedded Graphics Hello World Example](https://github.com/embedded-graphics/examples/blob/main/eg-0.8/examples/hello-world.rs).

![Image of the Embedded Graphics example](./images/embedded_graphics_example.jpg)


## Clock Example

[This example](./clock/) shows an incrementing clock.
The clock increments once per second, and is capable of counting up to 255 days.

![Image of the Clock Demo, showing the time "000:00:00:15", aka having counted 15 seconds](./images/clock_example.jpg)

