use raug::{
    prelude::*,
    runtime::{RuntimeError, RuntimeHandle},
};
use iced::{
    widget::{button, column},
    Application, Command,
};

use crate::widgets::Widget;

#[derive(Debug, Clone)]
pub enum IcedRuntimeMessage<M> {
    StartAudio,
    StopAudio,
    Message(M),
}

#[derive(Debug, thiserror::Error)]
pub enum IcedRuntimeError {
    #[error("Processor error: {0}")]
    Processor(#[from] RuntimeError),
    #[error("Iced error: {0}")]
    Iced(#[from] iced::Error),
}

pub struct IcedRuntime<T: Widget> {
    graph: Graph,
    main_widget: T,
}

impl<T: Widget> IcedRuntime<T> {
    pub fn new(graph: Graph, main_widget: T) -> Self {
        Self { graph, main_widget }
    }

    pub fn run(self, backend: Backend, device: Device) -> Result<(), IcedRuntimeError> {
        let Self { graph, main_widget } = self;

        let settings = iced::Settings {
            flags: (graph, main_widget, backend, device),
            id: None,
            window: Default::default(),
            default_font: None,
            default_text_size: 16.0,
            antialiasing: true,
            exit_on_close_request: true,
            text_multithreading: false,
            try_opengles_first: false,
        };

        IcedRuntimeApp::run(settings).map_err(IcedRuntimeError::Iced)
    }
}

pub struct IcedRuntimeApp<T: Widget> {
    runtime: Option<Runtime>,
    handle: Option<RuntimeHandle>,
    main_widget: T,
    running: bool,
    backend: Backend,
    device: Device,
}

impl<T: Widget> Application for IcedRuntimeApp<T> {
    type Executor = iced::executor::Default;
    type Message = IcedRuntimeMessage<T::Message>;
    type Theme = iced::theme::Theme;
    type Flags = (Graph, T, Backend, Device);

    fn new((graph, main_widget, backend, device): Self::Flags) -> (Self, Command<Self::Message>) {
        let runtime = Runtime::new(graph);
        (
            Self {
                runtime: Some(runtime),
                handle: None,
                main_widget,
                running: false,
                backend,
                device,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "raug".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            IcedRuntimeMessage::StartAudio => {
                if self.running {
                    return Command::none();
                }

                let mut runtime = self.runtime.take().unwrap();
                let handle = runtime
                    .run(self.backend.clone(), self.device.clone())
                    .unwrap();
                self.handle = Some(handle);
                self.running = true;
            }
            IcedRuntimeMessage::StopAudio => {
                if !self.running {
                    return Command::none();
                }

                let handle = self.handle.take().unwrap();
                handle.stop();
                self.running = false;
            }
            IcedRuntimeMessage::Message(message) => {
                self.main_widget.update(message);
            }
        }

        Command::none()
    }

    fn view(&self) -> iced::Element<Self::Message> {
        column![
            button("Start Audio").on_press(IcedRuntimeMessage::StartAudio),
            button("Stop Audio").on_press(IcedRuntimeMessage::StopAudio),
            self.main_widget.view().map(IcedRuntimeMessage::Message),
        ]
        .into()
    }
}
