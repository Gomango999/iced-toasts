use iced::{
    Element, Length,
    widget::{button, column, container, text},
};
use iced_toasts::{Id, Level, ToastManager};

pub fn main() -> iced::Result {
    iced::run("Toasts", App::update, App::view)
}

struct App<'a, Message> {
    toasts: ToastManager<'a, Message>,
    toast_counter: usize,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    PushToast,
    DismissToast(Id),
    ToastActioned(usize),
}

impl Default for App<'_, Message> {
    fn default() -> Self {
        Self {
            toasts: ToastManager::new(Message::DismissToast)
                .alignment_x(iced_toasts::alignment::Horizontal::Left)
                .alignment_y(iced_toasts::alignment::Vertical::Top),
            toast_counter: 0,
        }
    }
}

impl App<'_, Message> {
    fn update(&mut self, message: Message) {
        match message {
            Message::PushToast => {
                self.toasts.push_toast(
                    Level::Success,
                    "Success",
                    &format!("New Toast Added! ({:?})", self.toast_counter),
                    Some(("Undo", Message::ToastActioned(self.toast_counter))),
                );
                self.toast_counter += 1;
            }
            Message::DismissToast(id) => {
                self.toasts.dismiss_toast(id);
            }
            Message::ToastActioned(value) => {
                println!("Actioned! {value}")
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content = button(text("Add new toast!")).on_press(Message::PushToast);
        let content = container(column![content]).align_right(Length::Fill);
        self.toasts.view(content)
    }
}
