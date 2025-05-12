//! `iced_toasts` is an add-on crate to the [iced](https://iced.rs/) GUI library,
//! which provides a simple way to add toast notifications. It is inspired by
//! [`examples/toast`](https://github.com/iced-rs/iced/tree/master/examples/toast).
//!
//! # Features
//! In addition to the features of the example iced toast code, this create supports:
//!
//! - Optional title, level and action buttons
//! - Styling and positioning options
//! - Toasts will not automatically disappear if being actively hovered over
//!
//! # Example
//! Here is a minimal example to push toasts to the screen
//! ```rust
//! use iced_toasts::{ToastContainer, ToastId, ToastLevel, toast, toast_container};
//!
//! pub fn main() -> iced::Result {
//!     iced::run("Iced Toasts Example", App::update, App::view)
//! }
//!
//! struct App<'a, Message> {
//!     toasts: ToastContainer<'a, Message>,
//! }
//!
//! #[derive(Debug, Clone, Copy)]
//! enum Message {
//!     PushToast,
//!     DismissToast(ToastId),
//! }
//!
//! impl Default for App<'_, Message> {
//!     fn default() -> Self {
//!         Self {
//!             toasts: toast_container(Message::DismissToast),
//!         }
//!     }
//! }
//!
//! impl App<'_, Message> {
//!     fn update(&mut self, message: Message) {
//!         match message {
//!             Message::PushToast => {
//!                 self.toasts.push(
//!                     toast("Added a new toast!")
//!                         .title("Success")
//!                         .level(ToastLevel::Success),
//!                 );
//!             }
//!             Message::DismissToast(id) => {
//!                 self.toasts.dismiss(id);
//!             }
//!         }
//!     }
//!
//!     fn view(&self) -> Element<Message> {
//!         let toast_button = button(text("Add new toast!")).on_press(Message::PushToast);
//!         let content = container(toast_button);
//!         self.toasts.view(content)
//!     }
//! }
//! ```
//!
//! # Action Buttons
//! iced_toasts allows you to add an optional action button to each toast, which
//! will broadcast a user-defined message if pressed.
//!
//! ```ignore
//! enum Message {
//!     RemoveFile(usize),
//!     UndoFileRemoval(usize),
//! }
//!
//! fn update(&mut self, message: Message) {
//!     match message {
//!         RemoveFile(file_id) => {
//!             self.toasts.push(
//!                 toast(&format!("File removed ({})", file_id))
//!                 .level(ToastLevel::Success)
//!                 .action("Undo", Message::UndoFileRemoval(file_id))
//!             );
//!         },
//!         UndoFileRemoval(file_id) => {
//!             println!("File removal undone!")
//!         }
//!     }
//! ```
//!
//! # Styling
//! Toasts appear on the bottom right with rounded corners by default. We can
//! change the alignment and size using builder methods when initialising
//! [`ToastContainer`].
//!
//! ```rust
//! use iced_toasts::{toast_container, alignment, ToastId};
//!
//! enum Message {
//!     DismissToast(ToastId),
//! }
//!
//! let toasts = toast_container(Message::DismissToast)
//!     .alignment_x(alignment::Horizontal::Left)
//!     .alignment_y(alignment::Vertical::Bottom)
//!     .size(24);
//! ```
//!
//! For more fine tuned styling of the appearance of individual toasts, we can
//! call the `style` method. This behaves similarly to styles in iced, as it
//! takes a reference to a theme and returns the [`Style`] struct.
//!
//! ```rust
//! use iced_toasts::{toast_container, alignment, ToastId};
//!
//! enum Message {
//!     DismissToast(ToastId),
//! }
//!
//! let toasts = toast_container(Message::DismissToast)
//!     .style(|theme| {
//!         let palette = theme.extended_palette();
//!         iced_toasts::Style {
//!             text_color: Some(palette.background.base.text),
//!             background: None,
//!             border: Border::default(),
//!             shadow: Shadow::default(),
//!             level_to_color: Rc::new(|_level| None),
//!         }
//!     });
//! ```

use std::{cell::RefCell, cmp, rc::Rc};

use iced::{
    Background, Border, Color, Element, Event, Length, Padding, Pixels, Point, Rectangle, Renderer,
    Shadow, Size, Theme, Vector,
    advanced::{
        Clipboard, Layout, Shell, Widget,
        layout::{self, Limits, Node, flex::Axis},
        mouse::{self, Cursor, Interaction},
        overlay,
        renderer::{self},
        widget::{
            Operation, Tree,
            tree::{State, Tag},
        },
    },
    event, time, window,
};

mod toast;
pub use toast::Id as ToastId;
pub use toast::Level as ToastLevel;

pub mod alignment {
    //! This module provides some structs for choosing where toasts will display
    //! on-screen.

    /// The horizontal position of toasts on the screen
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

    /// The vertical position of toasts on the screen
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

mod toast_builder {
    use super::ToastLevel;

    #[derive(Default, Clone, Debug)]
    pub struct ToastBuilder<Message> {
        pub(crate) message: String,
        pub(crate) title: Option<String>,
        pub(crate) level: Option<ToastLevel>,
        pub(crate) action: Option<(String, Message)>,
    }

    /// Starts building a new toast with the provided message. Optional fields can
    /// be added using builder-style methods suhc as [`title`], [`level`], and [`action`]
    ///
    /// # Example
    /// ```rust
    /// use iced_toasts::{toast, ToastLevel};
    ///
    /// enum Message {
    ///     UndoFileCreation,
    /// }
    ///
    /// toast("New file created")
    ///     .title("Success")
    ///     .level(ToastLevel::Success)
    ///     .action("Undo", Message::UndoFileCreation);
    /// ```
    pub fn toast<Message>(message: &str) -> ToastBuilder<Message> {
        ToastBuilder {
            message: message.to_string(),
            title: None,
            level: None,
            action: None,
        }
    }

    impl<Message> ToastBuilder<Message> {
        /// Adds an optional title to `Toast`.
        ///
        /// If not set, the toast will have no title.
        pub fn title(mut self, title: &str) -> Self {
            self.title = Some(title.to_string());
            self
        }

        /// Adds an optional level to `Toast`.
        ///
        /// If not set, the toast will not have a colored border.
        pub fn level(mut self, level: ToastLevel) -> Self {
            self.level = Some(level);
            self
        }

        /// Adds an optional action button to `Toast`.
        ///
        /// `text`` is displayed on the button, and `message` is broadcast when
        /// the action button is pressed.
        pub fn action(mut self, text: &str, message: Message) -> Self {
            self.action = Some((text.to_string(), message));
            self
        }
    }
}

pub type Toast<Message> = toast_builder::ToastBuilder<Message>;
pub use toast_builder::toast;

/// A component responsible for managing the state of toasts. This should be
/// created using [`toast_container()`] during the initialisation of the
/// iced application.
///
/// # Example
/// ```rust
/// enum Message {
///     DismissToast(ToastId),
/// }
/// let toasts = toast_container(Message::DismissToast);
/// ```
pub struct ToastContainer<'a, Message> {
    toasts: Rc<RefCell<Vec<toast::Toast<Message>>>>,
    next_toast_id: ToastId,
    timeout_duration: time::Duration,
    on_dismiss: Rc<Box<dyn Fn(ToastId) -> Message + 'a>>,
    alignment_x: alignment::Horizontal,
    alignment_y: alignment::Vertical,
    text_size: Pixels,
    style_fn: StyleFn<'a>,
    // SOMEDAY: Add an option to disable extending the timeout when the mouse
    // is hovered over the toasts.
}

/// Creates a new [`ToastContainer`], which is responsible for managing the
/// state and appearance of toasts. You can configure [`ToastContainer`] by
/// chaining builder methods, and get the associated [`Element`] using
/// [`ToastContainer::view()`].
///
/// The message produced by `on_dismiss(ToastId)` is broadcasted whenever the user
/// clicks the dismiss button on a toast.
///
/// # Example
/// ```rust
/// enum Message {
///     DismissToast(ToastId),
/// }
///
/// let toasts = toast_container(Message::DismissToast);
/// ```
pub fn toast_container<'a, Message: 'a + Clone + std::fmt::Debug>(
    on_dismiss: impl Fn(ToastId) -> Message + 'a,
) -> ToastContainer<'a, Message> {
    ToastContainer::new(on_dismiss)
}

impl<'a, Message> ToastContainer<'a, Message>
where
    Message: 'a + Clone + std::fmt::Debug,
{
    fn new(on_dismiss: impl Fn(ToastId) -> Message + 'a) -> Self {
        ToastContainer {
            toasts: Rc::new(RefCell::new(Vec::new())),
            next_toast_id: ToastId::new(),
            timeout_duration: time::Duration::new(5, 0),
            on_dismiss: Rc::new(Box::new(on_dismiss)),
            alignment_x: alignment::Horizontal::Right,
            alignment_y: alignment::Vertical::Bottom,
            text_size: 16.into(),
            style_fn: StyleFn::default(),
        }
    }

    /// Sets the horizontal position at which the toasts will appear.
    pub fn alignment_x(mut self, alignment: alignment::Horizontal) -> Self {
        self.alignment_x = alignment;
        self
    }

    /// Sets the vertical position at which the toasts will appear.
    pub fn alignment_y(mut self, alignment: alignment::Vertical) -> Self {
        self.alignment_y = alignment;
        self
    }

    /// Sets the amount of time toasts have before they disappear. Default is 5
    /// seconds.
    pub fn timeout(mut self, timeout: time::Duration) -> Self {
        self.timeout_duration = timeout;
        self
    }

    /// Sets the text size of the toast. Default is 16.
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.text_size = size.into();
        self
    }

    /// Sets the style of the [`ToastContainer`].
    pub fn style(mut self, style_fn: impl Fn(&iced::Theme) -> Style + 'a) -> Self {
        self.style_fn = StyleFn(Rc::new(style_fn));
        self
    }

    /// Displays a new toast on-screen.
    pub fn push(&mut self, toast: Toast<Message>) {
        self.toasts.borrow_mut().push(toast::Toast {
            id: self.next_toast_id,
            expiry: time::Instant::now() + self.timeout_duration,
            level: toast.level,
            title: toast.title,
            message: toast.message,
            on_dismiss: (self.on_dismiss)(self.next_toast_id),
            action: toast
                .action
                .map(|(text, message)| (text.to_string(), message)),
        });
    }

    /// Dismisses a toast. Should be called with the corresponding [`ToastId`]
    /// whenever the `on_dismiss` message is received.
    pub fn dismiss(&mut self, id: ToastId) {
        self.toasts.borrow_mut().retain(|toast| toast.id != id);
    }

    /// Creates the [`Element`] for the [`ToastContainer`] to be used in the
    /// application view function. [`ToastContainer`] acts as a container, and
    /// the `content` passed in will display as normal, with any toasts overlaid
    /// on top.
    ///
    /// # Example
    /// In the view function of the application:
    /// ```ignore
    /// fn view(&self) -> Element<Message> {
    ///    let toast_button = button(text("Add new toast!")).on_press(Message::PushToast);
    ///    self.toasts.view(toast_button)
    /// }
    /// ```
    pub fn view(&self, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
        Element::new(ToastWidget::<'a, Message>::new(
            self.toasts.clone(),
            content,
            self.on_dismiss.clone(),
            self.alignment_x,
            self.alignment_y,
            self.text_size,
            self.style_fn.clone(),
        ))
    }
}

impl<'a, Message> std::fmt::Debug for ToastContainer<'a, Message>
where
    Message: 'a + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToastContainer")
            .field("toasts", &self.toasts)
            .field("next_toast_id", &self.next_toast_id)
            .field("timeout_duration", &self.timeout_duration)
            .field("alignment_x", &self.alignment_x)
            .field("alignment_y", &self.alignment_y)
            .field("text_size", &self.text_size)
            .finish()
    }
}

// The [`Widget`] reponsible for displaying toasts. It is responsible for expiring
// toasts at the correct time.
struct ToastWidget<'a, Message> {
    content: Element<'a, Message>,
    toasts: Rc<RefCell<Vec<toast::Toast<Message>>>>,
    toast_elements: Vec<Element<'a, Message>>,

    on_dismiss: Rc<Box<dyn Fn(ToastId) -> Message + 'a>>,

    alignment_x: alignment::Horizontal,
    alignment_y: alignment::Vertical,
}

impl<'a, Message> ToastWidget<'a, Message>
where
    Message: 'a + Clone,
{
    fn new(
        toasts: Rc<RefCell<Vec<toast::Toast<Message>>>>,
        content: impl Into<Element<'a, Message>>,
        on_dismiss: Rc<Box<dyn Fn(ToastId) -> Message + 'a>>,
        alignment_x: alignment::Horizontal,
        alignment_y: alignment::Vertical,
        text_size: Pixels,
        style_fn: StyleFn<'a>,
    ) -> Self {
        let mut toast_elements: Vec<_> = toasts
            .borrow()
            .iter()
            .map(|toast| toast.view(text_size, style_fn.clone()))
            .collect();
        if alignment_y == alignment::Vertical::Top {
            toast_elements.reverse()
        }

        ToastWidget {
            content: content.into(),
            toasts,
            toast_elements,
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
        style: &renderer::Style,
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
                .for_each(|&toast::Toast { id, expiry, .. }| {
                    if now > &expiry {
                        shell.publish((self.on_dismiss)(id));
                    } else {
                        // If we do not expire a toast now, we guarantee that
                        // there will be another redraw request at the time
                        // the toast expires, so that this handler will be
                        // called again at that time.
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
    toasts: Rc<RefCell<Vec<toast::Toast<Message>>>>,
    elements: &'b mut [Element<'a, Message>],
    state: &'b mut [Tree],

    position: Point,
    alignment_x: alignment::Horizontal,
    alignment_y: alignment::Vertical,
}

impl<'a, 'b, Message> Overlay<'a, 'b, Message> {
    fn new(
        toasts: Rc<RefCell<Vec<toast::Toast<Message>>>>,
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
        style: &renderer::Style,
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
                child
                    .as_widget()
                    .draw(state, renderer, theme, style, layout, cursor, &viewport)
            }
        }
        // SOMEDAY: Make toasts not draw if they cannot fit. Currently, they
        // just shrink in size and display some of it's elements. Perhaps
        // implement a queue system so that cut off toasts still have a
        // chance to display later.
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

/// Defines a mapping from [`ToastLevel`] to a color that will be used to display
/// on the border of toasts.
pub type LevelToColorMap<'a> = Rc<dyn Fn(&ToastLevel) -> Option<Color> + 'a>;

/// Defines the styles toasts created by a `ToastContainer`
#[derive(Clone)]
pub struct Style<'a> {
    /// The color of the toast text
    pub text_color: Option<Color>,
    /// The background of the toast
    pub background: Option<Background>,
    /// The border of the entire toast
    pub border: Border,
    /// The shadow of the toast
    pub shadow: Shadow,
    /// A mapping from [`ToastLevel`] to colors, which determine color
    /// the left-border of the toast
    pub level_to_color: LevelToColorMap<'a>,
}

impl<'a> Style<'a> {
    /// Updates the text color of the [`Style`].
    pub fn color(self, color: impl Into<Color>) -> Self {
        Self {
            text_color: Some(color.into()),
            ..self
        }
    }

    /// Updates the border of the [`Style`].
    pub fn border(self, border: impl Into<Border>) -> Self {
        Self {
            border: border.into(),
            ..self
        }
    }

    /// Updates the background of the [`Style`].
    pub fn background(self, background: impl Into<Background>) -> Self {
        Self {
            background: Some(background.into()),
            ..self
        }
    }

    /// Updates the shadow of the [`Style`].
    pub fn shadow(self, shadow: impl Into<Shadow>) -> Self {
        Self {
            shadow: shadow.into(),
            ..self
        }
    }

    /// Updates the mapping from toast levels to colors within [`Style`].
    pub fn level_to_color(
        self,
        level_to_color: impl Fn(&toast::Level) -> Option<Color> + 'a,
    ) -> Self {
        Self {
            level_to_color: Rc::new(level_to_color),
            ..self
        }
    }
}

impl std::fmt::Debug for Style<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Style")
            .field("text_color", &self.text_color)
            .field("background", &self.background)
            .field("border", &self.border)
            .field("shadow", &self.shadow)
            .finish()
    }
}

#[derive(Clone)]
struct StyleFn<'a>(Rc<dyn Fn(&iced::Theme) -> Style + 'a>);

impl<'a> Default for StyleFn<'a> {
    fn default() -> Self {
        StyleFn(Rc::new(|theme: &iced::Theme| {
            let palette = theme.extended_palette().clone();
            Style {
                text_color: Some(palette.background.base.text),
                background: Some(palette.background.base.color.into()),
                border: Border {
                    color: palette.background.base.text,
                    width: 1.0,
                    radius: 5.0.into(),
                },
                shadow: Shadow::default(),
                level_to_color: Rc::new(move |level: &toast::Level| match level {
                    toast::Level::Info => Some(palette.primary.strong.color),
                    toast::Level::Success => Some(palette.success.strong.color),
                    toast::Level::Warning => Some(palette.danger.strong.color),
                    toast::Level::Error => Some(palette.danger.strong.color),
                }),
            }
        }))
    }
}

pub mod style {
    use iced::Border;

    /// The default style function of a toast. This contains rounded corners and
    /// uses the colors defined in `theme`.
    pub fn default(theme: &iced::Theme) -> super::Style {
        super::StyleFn::default().0(theme)
    }

    /// Same as the default style, but with square corners.
    pub fn square_box(theme: &iced::Theme) -> super::Style {
        let palette = theme.extended_palette();

        let style = super::StyleFn::default().0(theme);
        style.border(Border {
            color: palette.background.base.text,
            width: 1.0,
            radius: 0.0.into(),
        })
    }
}
