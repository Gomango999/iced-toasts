#![allow(dead_code)]
use std::rc::Rc;

use iced::{
    Element, Event, Length, Padding, Point, Rectangle, Renderer, Size, Theme, Vector,
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
    event, time, window,
};

// TODO: Think about the API.
// E.g. Not everything in toast should be exposed, what derives should I expose, how should I name?
mod toast;
pub use toast::*;

pub mod alignment {

    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum Horizontal {
        Left,
        Center,
        Right,
    }

    impl Into<iced::alignment::Alignment> for Horizontal {
        fn into(self) -> iced::alignment::Alignment {
            match self {
                Horizontal::Left => iced::alignment::Horizontal::Left,
                Horizontal::Center => iced::alignment::Horizontal::Center,
                Horizontal::Right => iced::alignment::Horizontal::Right,
            }
            .into()
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum Vertical {
        Top,
        Bottom,
    }

    impl Into<iced::alignment::Alignment> for Vertical {
        fn into(self) -> iced::alignment::Alignment {
            match self {
                Vertical::Top => iced::alignment::Vertical::Top,
                Vertical::Bottom => iced::alignment::Vertical::Bottom,
            }
            .into()
        }
    }
}

pub struct ToastManager<'a, Message> {
    toasts: Vec<Toast<Message>>,
    next_toast_id: Id,
    timeout: time::Duration,
    on_dismiss: Rc<Box<dyn Fn(Id) -> Message + 'a>>,
    alignment_x: alignment::Horizontal,
    alignment_y: alignment::Vertical,
}

impl<'a, Message> ToastManager<'a, Message>
where
    Message: 'a + Clone + std::fmt::Debug,
{
    pub fn new(on_dismiss: impl Fn(Id) -> Message + 'a) -> Self {
        ToastManager {
            toasts: Vec::new(),
            next_toast_id: Id::new(),
            timeout: time::Duration::new(10, 0),
            on_dismiss: Rc::new(Box::new(on_dismiss)),
            alignment_x: alignment::Horizontal::Right,
            alignment_y: alignment::Vertical::Bottom,
        }
    }
    pub fn alignment_x(mut self, alignment: alignment::Horizontal) -> Self {
        self.alignment_x = alignment;
        self
    }
    pub fn alignment_y(mut self, alignment: alignment::Vertical) -> Self {
        self.alignment_y = alignment;
        self
    }
    pub fn timeout(mut self, timeout: time::Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn push_toast(
        &mut self,
        level: Level,
        title: &str,
        message: &str,
        action: Option<(&str, Message)>,
    ) {
        self.toasts.push(Toast {
            id: self.next_toast_id,
            created_at: time::Instant::now(),
            level,
            title: title.to_string(),
            message: message.to_string(),
            on_dismiss: (self.on_dismiss)(self.next_toast_id),
            action: action.map(|(text, message)| (text.to_string(), message)),
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
            self.timeout,
            self.on_dismiss.clone(),
            self.alignment_x,
            self.alignment_y,
        ))
    }
}

// TODO: Add styling options
pub struct ToastWidget<'a, Message> {
    content: Element<'a, Message>,
    toasts: Vec<Toast<Message>>,
    // `toast_elements[i]` is the corresponding element to `toasts[i]`.
    // We store them in two separate vectors instead of one because the overlay
    // requires a &[Toast] slice.
    toast_elements: Vec<Element<'a, Message>>,

    timeout: time::Duration,
    on_dismiss: Rc<Box<dyn Fn(Id) -> Message + 'a>>,

    alignment_x: alignment::Horizontal,
    alignment_y: alignment::Vertical,
}

impl<'a, Message: 'a + Clone> ToastWidget<'a, Message> {
    fn new(
        toasts: &Vec<Toast<Message>>,
        content: impl Into<Element<'a, Message>>,
        timeout_secs: time::Duration,
        on_dismiss: Rc<Box<dyn Fn(Id) -> Message + 'a>>,
        alignment_x: alignment::Horizontal,
        alignment_y: alignment::Vertical,
    ) -> Self {
        let mut toasts = toasts.clone();
        if alignment_y == alignment::Vertical::Top {
            toasts.reverse()
        }

        let toast_elements = toasts.iter().map(|toast| toast.into()).collect();

        ToastWidget {
            content: content.into(),
            toasts: toasts.clone(),
            toast_elements,
            timeout: timeout_secs,
            on_dismiss,
            alignment_x,
            alignment_y,
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
                    if created_at.elapsed() > self.timeout {
                        shell.publish((self.on_dismiss)(id));
                    } else {
                        let request = window::RedrawRequest::At(created_at + self.timeout);
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
            let toast_overlay = Overlay::new(
                &mut self.toast_elements,
                toast_state,
                layout.bounds().position() + translation,
                self.alignment_x,
                self.alignment_y,
            );
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

    position: Point,
    alignment_x: alignment::Horizontal,
    alignment_y: alignment::Vertical,
}

impl<'a, 'b, Message> Overlay<'a, 'b, Message> {
    fn new(
        toasts: &'b mut [Element<'a, Message>],
        state: &'b mut [Tree],
        position: Point,
        alignment_x: alignment::Horizontal,
        alignment_y: alignment::Vertical,
    ) -> Self {
        Overlay {
            toasts,
            state,
            position,
            alignment_x,
            alignment_y,
        }
    }
}

impl<'a, Message> overlay::Overlay<Message, Theme, Renderer> for Overlay<'a, '_, Message> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> Node {
        // TODO: Fix bug where if too many toasts are spammed, the bottom ones become too small.
        layout::flex::resolve(
            Axis::Vertical,
            renderer,
            &Limits::new(Size::ZERO, bounds),
            Length::Shrink,
            Length::Shrink,
            Padding::from(5),
            5.0,
            self.alignment_x.into(),
            self.toasts,
            self.state,
        )
        .translate(Vector::new(self.position.x, self.position.y))
        .align(self.alignment_x.into(), self.alignment_y.into(), bounds)
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
