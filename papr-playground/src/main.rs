use iced::Application;

pub struct PaprPlayground;

impl Application for PaprPlayground {
    type Executor = iced::executor::Default;
    type Message = ();
    type Flags = ();
    type Theme = iced::theme::Theme;

    fn new(_flags: ()) -> (PaprPlayground, iced::Command<Self::Message>) {
        (PaprPlayground, iced::Command::none())
    }

    fn title(&self) -> String {
        String::from("Papr Playground")
    }

    fn update(&mut self, _message: Self::Message) -> iced::Command<Self::Message> {
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        iced::widget::Text::new("Hello, world!").into()
    }
}

fn main() -> iced::Result {
    PaprPlayground::run(iced::Settings::default())
}
