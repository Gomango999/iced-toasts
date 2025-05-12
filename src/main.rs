use iced::{
    Element, Theme,
    widget::{button, text},
};
use iced_toasts::{ToastContainer, ToastId, ToastLevel, toast, toast_container};

pub fn main() -> iced::Result {
    iced::application("Toasts", App::update, App::view)
        .theme(App::theme)
        .run()
}

struct App<'a, Message> {
    toasts: ToastContainer<'a, Message>,
    toast_counter: usize,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    PushToast,
    DismissToast(ToastId),
    ToastActioned(usize),
}

impl Default for App<'_, Message> {
    fn default() -> Self {
        let toasts = toast_container(Message::DismissToast).style(iced_toasts::style::square_box);

        Self {
            toasts,
            toast_counter: 0,
        }
    }
}

impl App<'_, Message> {
    fn update(&mut self, message: Message) {
        match message {
            Message::PushToast => {
                // self.toasts.push(
                //     toast("File \"image (3).jpeg\" has been removed")
                //         .title("Success")
                //         .level(ToastLevel::Success)
                //         .action("Undo", Message::ToastActioned(0)),
                // );

                self.toasts.push(
                    toast("An iced toast notification add-on")
                        .title("Iced Toasts")
                        .level(ToastLevel::Info),
                );

                self.toasts.push(
                    toast("An iced toast notification add-on")
                        .title("Iced Toasts")
                        .level(ToastLevel::Success),
                );

                self.toasts.push(
                    toast("An iced toast notification add-on")
                        .title("Iced Toasts")
                        .level(ToastLevel::Error),
                );

                // self.toasts.push(
                //     toast(&format!("This is a toast! ({:?})", self.toast_counter))
                //         .title("Wow!")
                //         .level(ToastLevel::Success),
                // );
                // self.toast_counter += 1;

                // self.toasts.push(
                //     toast(&format!(
                //         "This toast has no title! ({:?})",
                //         self.toast_counter
                //     ))
                //     .level(ToastLevel::Error),
                // );
                // self.toast_counter += 1;

                // self.toasts.push(
                //     toast(&format!("Change the view to display a clic;able button with text, that returns the message! Again, the code wasn't too hard to write, so went pretty fast. Imagine `limits` as the hard window size, constrained in addition by the containers size. We call `limits.resolve()` with container width and height, as well as size of contents. In some respect, a button is ;ust a container layout-wise. If we are shrink in the cross axis, then we can take up as much height as we like (up to the limits of the row itself) ({:?})", self.toast_counter))
                //         .title("Lesson: Working backwards, I was able to build up a clear set of limitations",)
                //         .level(ToastLevel::Success)
                //         .action("Undo", Message::ToastActioned(self.toast_counter))
                // );
                // self.toast_counter += 1;
            }
            Message::DismissToast(id) => {
                self.toasts.dismiss(id);
            }
            Message::ToastActioned(value) => {
                println!("Actioned! {value}")
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let toast_button = button(text("Add new toast!")).on_press(Message::PushToast);
        self.toasts.view(toast_button)
    }

    fn theme(&self) -> Theme {
        Theme::CatppuccinLatte
    }
}
