#![allow(dead_code)]

use std::any::Any;

use iced::{
    Alignment, Color, Element, Event, Length, Padding, Rectangle, Renderer, Size, Theme,
    advanced::{
        Clipboard, Layout, Shell, Widget,
        layout::{self, Limits, Node, flex::Axis},
        mouse::{Cursor, Interaction},
        overlay,
        renderer::Style,
        widget::{
            Id, Operation, Tree,
            tree::{State, Tag},
        },
    },
    event, mouse,
    time::Duration,
    widget::{column, text},
    window::Event::RedrawRequested,
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
pub struct Toast {
    level: Level,
    message: String,
    // TODO: Add a button (closure produces a message)
}

impl Toast {
    pub fn new(level: Level, message: &str) -> Self {
        Toast {
            level,
            message: message.to_string(),
        }
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
        let example_toast1 = Toast::new(Level::Success, "Nice!");
        let example_toast2 = Toast::new(Level::Warning, "Nice 2!");
        let toasts: Vec<Toast> = vec![example_toast1, example_toast2];
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

impl<'a, Message: 'a> Widget<Message, Theme, Renderer> for Manager<'a, Message> {
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
        Tag::of::<Vec<Toast>>()
    }

    fn state(&self) -> State {
        // TODO: Initialise with the empty state
        let example_toast1 = Toast::new(Level::Success, "Nice!");
        let example_toast2 = Toast::new(Level::Warning, "Nice 2!");
        let toasts: Vec<Toast> = vec![example_toast1, example_toast2];
        State::new(toasts)
    }

    fn children(&self) -> Vec<Tree> {
        std::iter::once(Tree::new(&self.content))
            .chain(self.toast_elements.iter().map(Tree::new))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        // TODO: The matching algorithm for toasts seems a little suspicious,
        // we truncate off the end if there are less toasts. Check this is
        // fine? Otherwise we could try diff_children_custom.
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

        // Handle operations for pushing toasts
        let manager_id = Id::new(MANAGER_ID);
        operation.custom(&mut state.state, Some(&manager_id));
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
        // README: Unfortunately I might have to scrap this idea. I can't figure
        // out how to trigger this `RedrawRequested` event after an operation is
        // processed. This means that it is possible that we successfully add a
        // toast, but it won't show up until we send another event (e.g. we
        // move the mouse a little).

        if let Event::Window(RedrawRequested(_)) = event {
            // Update all toasts in state tree to match the state.
            let toasts = tree.state.downcast_ref::<Vec<Toast>>();
            let children = toasts
                .clone()
                .into_iter()
                .map(|toast| toast.clone().into())
                .map(|element: Element<'a, Message>| Tree::new(element))
                .collect::<Vec<Tree>>();

            tree.children.truncate(1);
            tree.children.extend(children);

            // TODO: Only invalidate layout if the toasts have changed
            shell.invalidate_layout();
            shell.invalidate_widgets();
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

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: iced::Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let (content_state, toast_state) = state.children.split_at_mut(1);
        let content_overlay = self.content.as_widget_mut().overlay(
            &mut content_state[0],
            layout,
            renderer,
            translation,
        );

        // TODO: Remove expired toasts

        let toasts = state.state.downcast_ref::<Vec<Toast>>();
        self.toast_elements = toasts
            .clone()
            .into_iter()
            .map(|toast| toast.into())
            .collect();

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

// Used by operations to identify the manager widget.
const MANAGER_ID: &str = "iced_toast_manager";

pub struct PushToastOperation {
    toast: Toast,
}

impl PushToastOperation {
    pub fn new(toast: Toast) -> Self {
        Self { toast }
    }
}

impl<Message> Operation<Message> for PushToastOperation {
    fn container(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        operate_on_children: &mut dyn FnMut(&mut dyn Operation<Message>),
    ) {
        operate_on_children(self);
    }

    fn custom(&mut self, state: &mut dyn Any, id: Option<&Id>) {
        let state = state.downcast_mut::<State>().unwrap();
        let toasts = state.downcast_mut::<Vec<Toast>>();

        let manager_id = Id::new(MANAGER_ID);
        if id == Some(&manager_id) {
            toasts.push(self.toast.clone());
        }
    }
}
