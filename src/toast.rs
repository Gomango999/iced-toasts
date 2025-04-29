use iced::{
    Alignment, Color, Element, Length, Padding, Theme, time,
    widget::{Space, button, column, container, row, text},
};

#[derive(Clone, Copy, Debug)]
pub enum Level {
    Info,
    Success,
    Warning,
    Error,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Id(usize);

impl Id {
    pub fn new() -> Self {
        Id(0)
    }

    pub fn next(&self) -> Id {
        Id(self.0 + 1)
    }
}

#[derive(Clone, Debug)]
pub struct Toast<Message> {
    pub id: Id,
    pub created_at: time::Instant,

    pub level: Level,
    pub title: String,
    pub message: String,

    pub on_dismiss: Message,
    pub action: Option<(String, Message)>,
}

impl<'a, Message> From<&Toast<Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(toast: &Toast<Message>) -> Self {
        let toast = toast.clone();

        let left_border: Element<Message> = container(Space::with_width(Length::Fill))
            .width(4)
            .height(Length::Fill)
            .style(move |theme: &Theme| {
                let palette = theme.extended_palette();
                let color = match toast.level {
                    Level::Info => palette.primary.strong.color,
                    Level::Success => palette.success.strong.color,
                    Level::Warning => palette.danger.strong.color,
                    Level::Error => palette.danger.strong.color,
                };
                container::background(color).border(iced::Border {
                    color,
                    width: 2.0,
                    radius: 5.0.into(),
                })
            })
            .into();

        let content = column![
            text(toast.title).font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..iced::Font::DEFAULT
            }),
            text(toast.message)
        ]
        .padding(Padding::from([5, 10]));

        let action_button: Element<'a, Message> = toast
            .action
            .map(|(button_text, message)| {
                container(
                    button(text(button_text))
                        .style(|theme: &Theme, status| {
                            let palette = theme.extended_palette();

                            let background = match status {
                                button::Status::Active => None,
                                button::Status::Hovered => Some(palette.background.weak.color),
                                button::Status::Pressed => Some(palette.background.strong.color),
                                button::Status::Disabled => None,
                            }
                            .map(iced::Background::Color);

                            button::Style {
                                background,
                                text_color: palette.primary.base.color,
                                border: iced::Border {
                                    color: Color::WHITE,
                                    width: 0.0,
                                    radius: 5.0.into(),
                                },
                                ..button::Style::default()
                            }
                        })
                        .on_press(message),
                )
                .align_y(Alignment::Center)
                .height(Length::Fill)
                .into()
            })
            .unwrap_or_else(|| Space::new(0, 0).into());

        let dismiss_button = container(
            // TODO: Get x button displaying a little higher, not centered vertically
            // TODO: Hover effect doesn't reach top and bottom of toast.
            button(text("×").size(28))
                .style(|theme: &Theme, status| {
                    let palette = theme.extended_palette();

                    let background = match status {
                        button::Status::Active => None,
                        button::Status::Hovered => Some(palette.background.weak.color),
                        button::Status::Pressed => Some(palette.background.strong.color),
                        button::Status::Disabled => None,
                    }
                    .map(iced::Background::Color);

                    button::Style {
                        background,
                        text_color: Color::WHITE,
                        ..button::Style::default()
                    }
                })
                .on_press(toast.on_dismiss),
        )
        .center(Length::Shrink)
        .center_y(Length::Fill);

        let toast_element: Element<'a, Message> =
            container(row![left_border, content, action_button, dismiss_button])
                .height(55)
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();

                    container::Style {
                        background: Some(palette.background.base.color.into()),
                        border: iced::Border {
                            color: palette.background.weak.color.into(),
                            width: 1.0,
                            radius: 5.0.into(),
                        },
                        ..container::Style::default()
                    }
                })
                .clip(true)
                .into();

        toast_element
        // toast_element.explain(Color::WHITE)
    }
}
