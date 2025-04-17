use iced::{
    Element, Task,
    advanced::widget::operate,
    widget::{button, column, text},
};
use iced_toasts::{Level, Manager, PushToastOperation, Toast};

pub fn main() -> iced::Result {
    iced::application("Toasts", Toasts::update, Toasts::view).run()
}

#[derive(Default)]
struct Toasts {}

#[derive(Debug, Clone)]
enum Message {
    PushToast,
}

impl Toasts {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PushToast => {
                let toast = Toast::new(Level::Success, "New Toast Added!");
                let operation = PushToastOperation::new(toast);
                operate::<Message>(operation)
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            Manager::new(
                button(text("............... Add new toast!")).on_press(Message::PushToast)
            ),
            button(text(".")).on_press(Message::PushToast),
        ]
        .into()
    }
}
