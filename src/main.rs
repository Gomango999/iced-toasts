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
                .alignment_y(iced_toasts::alignment::Vertical::Bottom),
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
                    &format!("Added a new toast! ({:?})", self.toast_counter),
                    Some(("Undo", Message::ToastActioned(self.toast_counter))),
                );
                self.toast_counter += 1;

                self.toasts.push_toast(
                    Level::Success,
                    "Lesson: Working backwards, I was able to build up a clear set of limitations",
                    &format!("Change the view to display a clickable button with text, that returns the message! Again, the code wasn't too hard to write, so went pretty fast. Imagine `limits` as the hard window size, constrained in addition by the containers size. We call `limits.resolve()` with container width and height, as well as size of contents. In some respect, a button is just a container layout-wise. If we are shrink in the cross axis, then we can take up as much height as we like (up to the limits of the row itself) ({:?})", self.toast_counter),
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
