#![allow(dead_code)]

use std::rc::Rc;

use iced::{
    Alignment, Color, Element, Event, Length, Padding, Rectangle, Renderer, Size, Theme,
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
    widget::{column, text},
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
struct Toast {
    // TODO: Replace this with a wrapper around usize, so Id is opaque.
    id: Id,
    created_at: time::Instant,

    level: Level,
    message: String,
    // TODO: Add a button (closure produces a message)
}

impl<'a, Message> From<&Toast> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(value: &Toast) -> Self {
        let content: Element<_> =
            column![text(value.level.to_string()), text(value.message.clone()),].into();
        content.explain(Color::WHITE)
    }
}

pub struct ToastManager<'a, Message> {
    toasts: Vec<Toast>,
    next_toast_id: Id,
    timeout_secs: time::Duration,
    on_dismiss: Rc<Box<dyn Fn(Id) -> Message + 'a>>,
}

impl<'a, Message> ToastManager<'a, Message>
where
    Message: 'a,
{
    pub fn new(on_dismiss: impl Fn(Id) -> Message + 'a) -> Self {
        ToastManager {
            toasts: Vec::new(),
            next_toast_id: Id::new(),
            timeout_secs: time::Duration::new(5, 0),
            on_dismiss: Rc::new(Box::new(on_dismiss)),
        }
    }

    pub fn push_toast(&mut self, level: Level, message: &str) {
        self.toasts.push(Toast {
            id: self.next_toast_id,
            created_at: time::Instant::now(),
            level,
            message: message.to_string(),
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
    toasts: Vec<Toast>,
    // `elements[i]` is the corresponding element to `toasts[i]`.
    // We store them in two separate vectors instead of one because the overlay
    // requires a &[Toast] slice.
    elements: Vec<Element<'a, Message>>,
    timeout_secs: time::Duration,
    on_dismiss: Rc<Box<dyn Fn(Id) -> Message + 'a>>,
}

impl<'a, Message: 'a> ToastWidget<'a, Message> {
    fn new(
        toasts: &Vec<Toast>,
        content: impl Into<Element<'a, Message>>,
        timeout_secs: time::Duration,
        on_dismiss: Rc<Box<dyn Fn(Id) -> Message + 'a>>,
    ) -> Self {
        let elements = toasts.iter().map(|toast| toast.into()).collect();
        ToastWidget {
            content: content.into(),
            toasts: toasts.clone(),
            elements,
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
            .chain(self.elements.iter().map(Tree::new))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(
            &std::iter::once(&self.content)
                .chain(self.elements.iter())
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
            layout.children().next().unwrap(),
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
            let toast_overlay = Overlay::new(&self.elements, toast_state);
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
    toasts: &'b [Element<'a, Message>],
    state: &'b mut [Tree],
}

impl<'a, 'b, Message> Overlay<'a, 'b, Message> {
    fn new(toasts: &'b [Element<'a, Message>], state: &'b mut [Tree]) -> Self {
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
            0.0,
            Alignment::End,
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
}
