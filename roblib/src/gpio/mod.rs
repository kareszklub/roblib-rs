use anyhow::Result;

pub mod cmd;
pub mod event;

#[cfg(feature = "gpio-backend")]
pub mod backend;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Input,
    Output,
}

pub trait Gpio {
    fn read_pin(&self, pin: u8) -> Result<bool>;
    fn write_pin(&self, pin: u8, value: bool) -> Result<()>;
    fn pwm(&self, pin: u8, hz: f64, cycle: f64) -> Result<()>;
    fn servo(&self, pin: u8, degree: f64) -> Result<()>;

    fn pin_mode(&self, pin: u8, mode: Mode) -> Result<()>;
}

pub trait TypedGpio<'p, S: Subscriber> {
    type O: OutputPin<S> + 'p;
    type I: InputPin<S> + 'p;
    type P: Pin<S> + 'p;

    fn output_pin(&'p self, pin: u8) -> Result<Self::O>;
    fn input_pin(&'p self, pin: u8) -> Result<Self::I>;
    fn pin(&'p self, pin: u8) -> Result<Self::P>;
}

pub trait Pin<S: Subscriber> {
    type O: OutputPin<S>;
    type I: InputPin<S>;

    fn get_pin(&self) -> u8;

    fn set_to_output(self) -> Result<Self::O>;
    fn set_to_input(self) -> Result<Self::I>;
}

pub trait Subscriber {
    fn handle_event(&mut self);
}

pub trait InputPin<S: Subscriber>: Pin<S> {
    type O: OutputPin<S>;
    type P: Pin<S>;

    fn subscribe(&self, sub: S) -> Result<()>;

    fn read(&self) -> Result<bool>;

    fn set_to_output(self) -> Result<<Self as InputPin<S>>::O>;
    fn set_to_pin(self) -> Result<Self::P>;
}

pub trait OutputPin<S: Subscriber>: Pin<S> {
    type I: InputPin<S>;
    type P: Pin<S>;

    fn set(&mut self, value: bool) -> Result<()>;
    fn pwm(&mut self, hz: f64, cycle: f64) -> Result<()>;
    fn servo(&mut self, degree: f64) -> Result<()>;

    fn set_to_input(self) -> Result<<Self as OutputPin<S>>::I>;
    fn set_to_pin(self) -> Result<Self::P>;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait GpioAsync<'p> {
    type O: OutputPinAsync + 'p;
    type I: InputPinAsync + 'p;
    type P: PinAsync + 'p;

    async fn output_pin(&'p self, pin: u8) -> Result<Self::O>;
    async fn input_pin(&'p self, pin: u8) -> Result<Self::I>;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait PinAsync {
    type O: OutputPinAsync;
    type I: InputPinAsync;

    async fn get_pin(&self) -> u8;

    async fn set_to_output(self) -> Result<Self::O>;
    async fn set_to_input(self) -> Result<Self::I>;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait InputPinAsync: PinAsync {
    type O: OutputPinAsync;
    type P: PinAsync;

    async fn read(&self) -> Result<bool>;

    async fn set_to_output(self) -> Result<<Self as InputPinAsync>::O>;
    async fn set_to_pin(self) -> Result<Self::P>;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait OutputPinAsync: PinAsync {
    type I: InputPinAsync;
    type P: PinAsync;

    async fn set(&mut self, value: bool) -> Result<()>;
    async fn pwm(&mut self, hz: f64, cycle: f64) -> Result<()>;
    async fn servo(&mut self, degree: f64) -> Result<()>;

    async fn set_to_input(self) -> Result<<Self as OutputPinAsync>::I>;
    async fn set_to_pin(self) -> Result<Self::P>;
}
