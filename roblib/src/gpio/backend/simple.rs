use std::collections::HashMap;

use rppal::gpio::{InputPin, OutputPin};

enum Pin {
    Input(InputPin),
    Output(OutputPin),
}

pub struct SimpleGpioBackend {
    pins: HashMap<u8, Pin>,
    gpio: rppal::gpio::Gpio,
}

impl SimpleGpioBackend {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            gpio: rppal::gpio::Gpio::new()?,
            pins: HashMap::new(),
        })
    }
}
