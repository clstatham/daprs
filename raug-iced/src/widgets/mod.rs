use std::fmt::Debug;

use crate::{
    builder::IcedGraphBuilder,
    channel::{GuiChannel, GuiRx},
};
use raug::prelude::*;
use iced::{widget::*, Element};
use iced_audio::{Normal, NormalParam};

pub trait Widget: 'static {
    type Message: Send + Sync + Debug + Clone + 'static;
    fn view(&self) -> Element<Self::Message>;
    fn update(&mut self, message: Self::Message);
    fn rx(&self) -> GuiRx;
}

pub struct Button {
    label: String,
    channel: GuiChannel,
}

impl Button {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            channel: GuiChannel::new(),
        }
    }
}

impl Widget for Button {
    type Message = ();
    fn view(&self) -> Element<()> {
        button(self.label.as_str()).on_press(()).into()
    }

    fn update(&mut self, _message: ()) {
        self.channel.tx().send(Message::Bang);
    }

    fn rx(&self) -> GuiRx {
        self.channel.rx().clone()
    }
}

#[derive(Default)]
pub struct Knob {
    channel: GuiChannel,
    normal_param: NormalParam,
    tick_marks: iced_audio::tick_marks::Group,
}

impl Knob {
    pub fn new() -> Self {
        Self {
            channel: GuiChannel::new(),
            normal_param: NormalParam::default(),
            tick_marks: iced_audio::tick_marks::Group::default(),
        }
    }
}

impl Widget for Knob {
    type Message = Normal;
    fn view(&self) -> Element<Normal> {
        iced_audio::Knob::new(self.normal_param, |value| value)
            .tick_marks(&self.tick_marks)
            .into()
    }

    fn update(&mut self, message: Normal) {
        self.normal_param.update(message);
        self.channel
            .tx()
            .send(Message::Float(message.as_f32() as f64));
    }

    fn rx(&self) -> GuiRx {
        self.channel.rx().clone()
    }
}

impl IcedGraphBuilder {
    pub fn knob(&self) -> (Knob, Node) {
        self.add_widget(Knob::new())
    }
}

#[derive(Default)]
pub struct NumberDialer {
    channel: GuiChannel,
    value: f64,
}

impl NumberDialer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget for NumberDialer {
    type Message = f64;
    fn view(&self) -> Element<f64> {
        let value = self.value.to_string();
        TextInput::new("", &value).into()
    }

    fn update(&mut self, message: f64) {
        self.value = message;
        self.channel.tx().send(Message::Float(message));
    }

    fn rx(&self) -> GuiRx {
        self.channel.rx().clone()
    }
}

impl IcedGraphBuilder {
    pub fn number_dialer(&self) -> (NumberDialer, Node) {
        self.add_widget(NumberDialer::new())
    }
}
