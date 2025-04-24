use iced::{
    Element,
    widget::{button, text},
};
// TODO: Think about this interface
use iced_toasts::{Id, Level, ToastManager};

pub fn main() -> iced::Result {
    iced::run("Toasts", App::update, App::view)
}

struct App<'a, Message> {
    toasts: ToastManager<'a, Message>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    PushToast,
    DismissToast(Id),
}

impl Default for App<'_, Message> {
    fn default() -> Self {
        Self {
            toasts: ToastManager::new(Message::DismissToast),
        }
    }
}

impl App<'_, Message> {
    fn update(&mut self, message: Message) {
        match message {
            Message::PushToast => {
                self.toasts.push_toast(Level::Success, "New Toast Added!");
            }
            Message::DismissToast(id) => {
                self.toasts.dismiss_toast(id);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content = button(text("..........Add new toast!")).on_press(Message::PushToast);
        self.toasts.view(content).into()
    }
}
