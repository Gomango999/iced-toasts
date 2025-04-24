#![allow(dead_code)]

use std::rc::Rc;

use iced::{
    Alignment, Color, Element, Event, Length, Padding, Point, Rectangle, Renderer, Size, Theme,
    advanced::{
        Clipboard, Layout, Shell, Widget,
        layout::{self, Limits, Node, flex::Axis},
        mouse::{self, Cursor, Interaction},
        overlay,
        renderer::Style,
        widget::{
            Operation, Tree,
            tree::{State, Tag},
        },
    },
    event, time,
    widget::{Space, button, column, container, row, text},
    window,
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
    fn new() -> Self {
        Id(0)
    }

    fn next(&self) -> Id {
        Id(self.0 + 1)
    }
}

#[derive(Clone, Debug)]
struct Toast<Message> {
    // TODO: Replace this with a wrapper around usize, so Id is opaque.
    id: Id,
    created_at: time::Instant,

    level: Level,
    title: String,
    message: String,

    on_dismiss: Message,
    // TODO: Add a button (closure produces a message)
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

        let dismiss_button = container(
            // TODO: Get x button displaying a little higher, not centered vertically
            // TODO: Hover effect doesn't reach top and bottom of toast.
            button(text("×").size(28))
                .style(|theme: &Theme, status: button::Status| {
                    let palette = theme.extended_palette();

                    let background = match status {
                        button::Status::Active => None,
                        button::Status::Hovered => Some(palette.background.weak.color),
                        button::Status::Pressed => Some(palette.background.strong.color),
                        button::Status::Disabled => None,
                    }
                    .map(iced::Background::Color);

                    iced::widget::button::Style {
                        background,
                        text_color: Color::WHITE,
                        ..iced::widget::button::Style::default()
                    }
                })
                .on_press(toast.on_dismiss),
        )
        .center(Length::Shrink)
        .center_y(Length::Fill);

        container(row![left_border, content, dismiss_button])
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
            .into()
    }
}

pub struct ToastManager<'a, Message> {
    toasts: Vec<Toast<Message>>,
    next_toast_id: Id,
    timeout_secs: time::Duration,
    on_dismiss: Rc<Box<dyn Fn(Id) -> Message + 'a>>,
}

impl<'a, Message> ToastManager<'a, Message>
where
    Message: 'a + Clone + std::fmt::Debug,
{
    pub fn new(on_dismiss: impl Fn(Id) -> Message + 'a) -> Self {
        ToastManager {
            toasts: Vec::new(),
            next_toast_id: Id::new(),
            timeout_secs: time::Duration::new(10, 0),
            on_dismiss: Rc::new(Box::new(on_dismiss)),
        }
    }

    pub fn push_toast(&mut self, level: Level, title: &str, message: &str) {
        self.toasts.push(Toast {
            id: self.next_toast_id,
            created_at: time::Instant::now(),
            level,
            title: title.to_string(),
            message: message.to_string(),
            on_dismiss: (self.on_dismiss)(self.next_toast_id),
        });

        self.next_toast_id = self.next_toast_id.next();
    }

    pub fn dismiss_toast(&mut self, id: Id) {
        self.toasts.retain(|toast| toast.id != id);
    }

    pub fn view(&self, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
        Element::new(ToastWidget::new(
            &self.toasts,
            content,
            self.timeout_secs,
            self.on_dismiss.clone(),
        ))
    }
}

// TODO: Add styling options
pub struct ToastWidget<'a, Message> {
    content: Element<'a, Message>,
    toasts: Vec<Toast<Message>>,
    // `elements[i]` is the corresponding element to `toasts[i]`.
    // We store them in two separate vectors instead of one because the overlay
    // requires a &[Toast] slice.
    toast_elements: Vec<Element<'a, Message>>,
    timeout_secs: time::Duration,
    on_dismiss: Rc<Box<dyn Fn(Id) -> Message + 'a>>,
}

impl<'a, Message: 'a + Clone> ToastWidget<'a, Message> {
    fn new(
        toasts: &Vec<Toast<Message>>,
        content: impl Into<Element<'a, Message>>,
        timeout_secs: time::Duration,
        on_dismiss: Rc<Box<dyn Fn(Id) -> Message + 'a>>,
    ) -> Self {
        let elements = toasts.iter().map(|toast| toast.into()).collect();
        ToastWidget {
            content: content.into(),
            toasts: toasts.clone(),
            toast_elements: elements,
            timeout_secs,
            on_dismiss,
        }
    }
}

impl<Message> Widget<Message, Theme, Renderer> for ToastWidget<'_, Message> {
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        layout::contained(limits, Length::Shrink, Length::Shrink, |limits| {
            self.content
                .as_widget()
                .layout(&mut tree.children[0], renderer, limits)
        })
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        )
    }

    fn tag(&self) -> Tag {
        Tag::stateless()
    }

    fn state(&self) -> State {
        State::None
    }

    fn children(&self) -> Vec<Tree> {
        std::iter::once(Tree::new(&self.content))
            .chain(self.toast_elements.iter().map(Tree::new))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(
            &std::iter::once(&self.content)
                .chain(self.toast_elements.iter())
                .collect::<Vec<_>>(),
        );
    }

    fn operate(
        &self,
        state: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.content
                .as_widget()
                .operate(&mut state.children[0], layout, renderer, operation);
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        if let Event::Window(window::Event::RedrawRequested(_)) = &event {
            self.toasts
                .iter()
                .for_each(|&Toast { id, created_at, .. }| {
                    if created_at.elapsed() > self.timeout_secs {
                        shell.publish((self.on_dismiss)(id));
                    } else {
                        let request = window::RedrawRequest::At(created_at + self.timeout_secs);
                        shell.request_redraw(request);
                    }
                });
        }

        self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> Interaction {
        self.content.as_widget().mouse_interaction(
            &state.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'a>(
        &'a mut self,
        state: &'a mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: iced::Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        let (content_state, toast_state) = state.children.split_at_mut(1);
        let content_overlay = self.content.as_widget_mut().overlay(
            &mut content_state[0],
            layout,
            renderer,
            translation,
        );

        let toast_overlay = (!self.toasts.is_empty()).then(|| {
            let toast_overlay = Overlay::new(&mut self.toast_elements, toast_state);
            overlay::Element::new(Box::new(toast_overlay))
        });

        let overlays = content_overlay
            .into_iter()
            .chain(toast_overlay)
            .collect::<Vec<_>>();
        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

struct Overlay<'a, 'b, Message> {
    toasts: &'b mut [Element<'a, Message>],
    state: &'b mut [Tree],
}

impl<'a, 'b, Message> Overlay<'a, 'b, Message> {
    fn new(toasts: &'b mut [Element<'a, Message>], state: &'b mut [Tree]) -> Self {
        Overlay { toasts, state }
    }
}

impl<'a, Message> overlay::Overlay<Message, Theme, Renderer> for Overlay<'a, '_, Message> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> Node {
        layout::flex::resolve(
            Axis::Vertical,
            renderer,
            &Limits::new(Size::ZERO, bounds),
            Length::Shrink,
            Length::Shrink,
            Padding::from(0),
            5.0,
            Alignment::Start,
            self.toasts,
            self.state,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor: Cursor,
    ) {
        let viewport = layout.bounds();

        for ((child, state), layout) in self
            .toasts
            .iter()
            .zip(self.state.iter())
            .zip(layout.children())
        {
            child
                .as_widget()
                .draw(state, renderer, theme, style, layout, cursor, &viewport)
        }
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> iced::event::Status {
        let viewport = layout.bounds();

        self.toasts
            .iter_mut()
            .zip(self.state.iter_mut())
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    &viewport,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.toasts
            .iter()
            .zip(self.state.iter())
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn is_over(&self, layout: Layout<'_>, _renderer: &Renderer, cursor_position: Point) -> bool {
        layout
            .children()
            .any(|layout| layout.bounds().contains(cursor_position))
    }
}
