use iced::{
    Element,
    widget::{button, text},
};
use iced_toasts::{Level, ToastManager};

pub fn main() -> iced::Result {
    iced::run("Toasts", App::update, App::view)
}

struct App {
    toasts: ToastManager,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    PushToast,
}

impl Default for App {
    fn default() -> Self {
        Self {
            toasts: ToastManager::new(),
        }
    }
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::PushToast => {
                self.toasts.push_toast(Level::Success, "New Toast Added!");
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content = button(text("..........Add new toast!")).on_press(Message::PushToast);
        self.toasts.view(content).into()
    }
}
