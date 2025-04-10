#![allow(dead_code)]

use iced::{
    Alignment, Color, Element, Length, Padding, Rectangle, Renderer, Size, Theme,
    advanced::{
        Layout, Widget,
        layout::{self, Limits, Node, flex::Axis},
        mouse::{Cursor, Interaction},
        overlay,
        renderer::Style,
        widget::{
            Operation, Tree,
            tree::{State, Tag},
        },
    },
    time::Duration,
    widget::{column, text},
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

#[derive(Clone, Debug)]
struct Toast {
    level: Level,
    message: String,
    // TODO: Add a button (closure produces a message)
}

impl Toast {
    pub fn new(level: Level, message: String) -> Self {
        Toast { level, message }
    }
}

impl<'a, Message> From<Toast> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(value: Toast) -> Self {
        let content: Element<_> =
            column![text(value.level.to_string()), text(value.message),].into();
        content.explain(Color::WHITE)
    }
}

// TODO: Add styling options
pub struct Manager<'a, Message> {
    content: Element<'a, Message>,
    // Synced to match the toasts stored in the widget state.
    toast_elements: Vec<Element<'a, Message>>,
    duration: Duration,
}

impl<'a, Message: 'a> Manager<'a, Message> {
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        // TODO: Remove this and initialise with the empty vec
        let example_toast = Toast::new(Level::Success, "Nice!".to_string());
        let toasts: Vec<Toast> = vec![example_toast];
        let toast_elements: Vec<Element<'a, Message>> = toasts
            .clone()
            .into_iter()
            .map(|toast| toast.into())
            .collect();

        Manager {
            content: content.into(),
            toast_elements,
            duration: Duration::new(5, 0),
        }
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Manager<'_, Message> {
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        self.content
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
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
        Tag::of::<Vec<Toast>>()
    }

    fn state(&self) -> State {
        // TODO: Initialise with the empty state
        let example_toast = Toast::new(Level::Success, "Nice!".to_string());
        let toasts: Vec<Toast> = vec![example_toast];
        State::Some(Box::new(toasts))
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
        // TODO: Update this to consume push_toast operations (whatever that means)
        operation.container(None, layout.bounds(), &mut |operation| {
            self.content
                .as_widget()
                .operate(&mut state.children[0], layout, renderer, operation);
        });
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

        // TODO: Remove expired toasts
        // TODO: Update self.toast_elements based on state.

        let toast_overlay = (!self.toast_elements.is_empty()).then(|| {
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

impl<'a, Message> From<Manager<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(manager: Manager<'a, Message>) -> Self {
        Element::new(manager)
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

pub fn push_toast() -> () {
    // Creates an operation task, which updates the widget state
    todo!()
}
