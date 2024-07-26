use papr::{
    prelude::Process,
    signal::{Buffer, SignalRate},
};

pub trait Ui {
    fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response;
}

pub struct UiNode {
    pub ui: Box<dyn Ui>,
}

impl UiNode {
    pub fn new_from_boxed(ui: Box<dyn Ui>) -> Self {
        Self { ui }
    }

    pub fn new<T: Ui + 'static>(ui: T) -> Self {
        Self::new_from_boxed(Box::new(ui))
    }
}
