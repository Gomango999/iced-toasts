#![allow(dead_code)]
use std::{cell::RefCell, cmp, rc::Rc};

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
    toasts: Rc<RefCell<Vec<Toast<Message>>>>,
    next_toast_id: Id,
    timeout_duration: time::Duration,
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
            toasts: Rc::new(RefCell::new(Vec::new())),
            next_toast_id: Id::new(),
            timeout_duration: time::Duration::new(5, 0),
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
        self.timeout_duration = timeout;
        self
    }

    pub fn push_toast(
        &mut self,
        level: Level,
        title: &str,
        message: &str,
        action: Option<(&str, Message)>,
    ) {
        self.toasts.borrow_mut().push(Toast {
            id: self.next_toast_id,
            expiry: time::Instant::now() + self.timeout_duration,
            level,
            title: title.to_string(),
            message: message.to_string(),
            on_dismiss: (self.on_dismiss)(self.next_toast_id),
            action: action.map(|(text, message)| (text.to_string(), message)),
        });

        self.next_toast_id = self.next_toast_id.next();
    }

    pub fn dismiss_toast(&mut self, id: Id) {
        self.toasts.borrow_mut().retain(|toast| toast.id != id);
    }

    pub fn view(&self, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
        Element::new(ToastWidget::new(
            self.toasts.clone(),
            content,
            self.timeout_duration,
            self.on_dismiss.clone(),
            self.alignment_x,
            self.alignment_y,
        ))
    }
}

// TODO: Add styling options
pub struct ToastWidget<'a, Message> {
    content: Element<'a, Message>,
    toasts: Rc<RefCell<Vec<Toast<Message>>>>,
    toast_elements: Vec<Element<'a, Message>>,

    timeout: time::Duration,
    on_dismiss: Rc<Box<dyn Fn(Id) -> Message + 'a>>,

    alignment_x: alignment::Horizontal,
    alignment_y: alignment::Vertical,
}

impl<'a, Message: 'a + Clone> ToastWidget<'a, Message> {
    fn new(
        toasts: Rc<RefCell<Vec<Toast<Message>>>>,
        content: impl Into<Element<'a, Message>>,
        timeout_secs: time::Duration,
        on_dismiss: Rc<Box<dyn Fn(Id) -> Message + 'a>>,
        alignment_x: alignment::Horizontal,
        alignment_y: alignment::Vertical,
    ) -> Self {
        let mut toast_elements: Vec<_> = toasts.borrow().iter().map(|toast| toast.into()).collect();
        if alignment_y == alignment::Vertical::Top {
            toast_elements.reverse()
        }

        ToastWidget {
            content: content.into(),
            toasts,
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
            layout.children().next().unwrap(),
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
        if let Event::Window(window::Event::RedrawRequested(now)) = &event {
            self.toasts
                .borrow()
                .iter()
                .for_each(|&Toast { id, expiry, .. }| {
                    if now > &expiry {
                        shell.publish((self.on_dismiss)(id));
                    } else {
                        let request = window::RedrawRequest::At(expiry);
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

        let toast_overlay = (!self.toasts.borrow().is_empty()).then(|| {
            let toast_overlay = Overlay::new(
                self.toasts.clone(),
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
    toasts: Rc<RefCell<Vec<Toast<Message>>>>,
    elements: &'b mut [Element<'a, Message>],
    state: &'b mut [Tree],

    position: Point,
    alignment_x: alignment::Horizontal,
    alignment_y: alignment::Vertical,
}

impl<'a, 'b, Message> Overlay<'a, 'b, Message> {
    fn new(
        toasts: Rc<RefCell<Vec<Toast<Message>>>>,
        elements: &'b mut [Element<'a, Message>],
        state: &'b mut [Tree],
        position: Point,
        alignment_x: alignment::Horizontal,
        alignment_y: alignment::Vertical,
    ) -> Self {
        Overlay {
            toasts,
            elements,
            state,
            position,
            alignment_x,
            alignment_y,
        }
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
            Padding::from(5),
            5.0,
            self.alignment_x.into(),
            self.elements,
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

        // Reverse the iterator depending on whether the toasts display at the top
        // of the screen or the bottom. Ideally, I'd just reverse the iterator only
        // but I can't since zips can't be reversed, and the iterators are
        // different types, so you get this ugly piece of code duplication.
        if self.alignment_y == alignment::Vertical::Bottom {
            let toast_iterator = self
                .elements
                .iter()
                .rev()
                .zip(self.state.iter().rev())
                .zip(layout.children().rev());
            for ((child, state), layout) in toast_iterator {
                child
                    .as_widget()
                    .draw(state, renderer, theme, style, layout, cursor, &viewport)
            }
        } else {
            let toast_iterator = self
                .elements
                .iter()
                .zip(self.state.iter())
                .zip(layout.children());
            for ((child, state), layout) in toast_iterator {
                // Do not draw toasts that will be displayed offscreen. We would
                // ideally also pause the expiration timer on toasts that are not
                // being displayed, but for now, we'll just chalk it up to users
                // misusing the notification system if they are actually generating
                // so many toasts that they go off-screen.
                if layout.bounds().height < TOAST_HEIGHT {
                    continue;
                }

                child
                    .as_widget()
                    .draw(state, renderer, theme, style, layout, cursor, &viewport)
            }
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
        // This function will always be called right before
        // `ToastWidget::on_event`. This means that right before a toast is
        // considered for expiry as part of as `RedrawRequested`` event, we
        // will always be able to check if we are hovering the toasts and update
        // expiry time before the toast actually expires.
        let is_hovering_toasts = if let mouse::Cursor::Available(cursor_position) = cursor {
            self.is_over(layout, renderer, cursor_position)
        } else {
            false
        };
        if is_hovering_toasts {
            self.toasts.borrow_mut().iter_mut().for_each(|toast| {
                let now = time::Instant::now();
                let hover_timeout = time::Duration::new(2, 0);
                toast.expiry = cmp::max(toast.expiry, now + hover_timeout)
            })
        }

        let viewport = layout.bounds();

        self.elements
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
        self.elements
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
