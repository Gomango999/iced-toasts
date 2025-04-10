use iced::{Element, widget::horizontal_rule};
use iced_toasts::Manager;

pub fn main() -> iced::Result {
    iced::run("Toasts", Toasts::update, Toasts::view)
}

#[derive(Default)]
struct Toasts {}

#[derive(Debug, Clone, Copy)]
enum Message {}

impl Toasts {
    fn update(&mut self, _message: Message) {}

    fn view(&self) -> Element<Message> {
        Manager::new(horizontal_rule(20)).into()
    }
}
