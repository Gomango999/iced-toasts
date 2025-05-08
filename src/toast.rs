use iced::{
    Alignment, Border, Color, Element, Length, Padding, Pixels, Theme,
    border::Radius,
    time,
    widget::{Space, button, column, container, row, scrollable, text},
};

mod left_border;
use left_border::left_border;

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
    pub expiry: time::Instant,

    pub level: Option<Level>,
    pub title: Option<String>,
    pub message: String,

    // SOMEDAY: Support having multiple action buttons
    pub action: Option<(String, Message)>,
    pub on_dismiss: Message,
}

impl<'a, Message> Toast<Message>
where
    Message: 'a + Clone,
{
    pub(crate) fn view(
        &self,
        text_size: Pixels,
        style_fn: super::StyleFn<'a>,
    ) -> Element<'a, Message> {
        let toast = self.clone();

        let content: Element<Message> = {
            let style_fn_title = style_fn.clone().0;
            let title: Element<Message> = toast
                .title
                .map(|title| {
                    text(title)
                        .font(iced::Font {
                            weight: iced::font::Weight::Bold,
                            ..iced::Font::DEFAULT
                        })
                        .style(move |theme| {
                            let toast_style = style_fn_title(theme);
                            text::Style {
                                color: toast_style.text_color,
                            }
                        })
                        .size(text_size)
                        .into()
                })
                .unwrap_or(Space::new(0, 0).into());

            let style_fn_message = style_fn.clone().0;
            let message = text(toast.message)
                .style(move |theme| {
                    let toast_style = style_fn_message(theme);
                    text::Style {
                        color: toast_style.text_color,
                    }
                })
                .size(text_size);

            container(scrollable(
                column![title, message].padding(Padding::default().right(10)),
            ))
            .max_width(500)
            .height(Length::Shrink)
            .padding(Padding {
                top: 10.0,
                right: 10.0,
                bottom: 10.0,
                left: 20.0,
            })
            .into()
        };

        let action_button: Element<'a, Message> = toast
            .action
            .map(|(button_str, message)| {
                container(
                    button(text(button_str).size(text_size))
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

        let dismiss_button: Element<Message> = container(
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
                        text_color: palette.background.base.text,
                        border: iced::Border {
                            color: Color::TRANSPARENT,
                            width: 0.0,
                            radius: 5.0.into(),
                        },
                        ..button::Style::default()
                    }
                })
                .width(40)
                .on_press(toast.on_dismiss),
        )
        .center_y(Length::Fill)
        .into();

        let right_padding = Space::new(4, Length::Fixed(55.0));

        let style_fn_left_border = style_fn.clone().0;
        let style_fn_container = style_fn.clone().0;
        let toast_element: Element<Message> = container(
            left_border(
                row![content, action_button, dismiss_button, right_padding].height(Length::Shrink),
            )
            .style(move |theme| {
                let toast_style = style_fn_left_border(theme);

                let color = toast
                    .level
                    .map(|level| (toast_style.level_to_color)(&level).unwrap_or(Color::TRANSPARENT))
                    .unwrap_or(Color::TRANSPARENT);

                Border {
                    color,
                    width: 3.0,
                    radius: Radius {
                        top_left: toast_style.border.radius.top_left,
                        top_right: 0.0,
                        bottom_right: 0.0,
                        bottom_left: toast_style.border.radius.bottom_right,
                    },
                }
            }),
        )
        .max_height(240)
        .style(move |theme: &Theme| {
            let toast_style = style_fn_container(theme);
            container::Style {
                text_color: toast_style.text_color,
                background: toast_style.background,
                border: toast_style.border,
                shadow: toast_style.shadow,
            }
        })
        .clip(true)
        .into();

        toast_element
    }
}
