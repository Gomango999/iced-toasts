//! This module defines a custom [`LeftBorder`] widget which defines a container
//! like widget with a left-border.

use std::rc::Rc;

use iced::{
    Border, Element, Event, Length, Rectangle, Renderer, Shadow, Size, Theme, Vector,
    advanced::{
        Clipboard, Layout, Shell, Widget, layout, renderer,
        widget::{Operation, Tree, tree},
    },
    mouse, overlay,
};

pub struct LeftBorder<'a, Message> {
    width: Length,
    height: Length,
    content: Element<'a, Message, Theme, Renderer>,
    style_fn: StyleFn<'a>,
}

pub fn left_border<'a, Message>(
    content: impl Into<Element<'a, Message>>,
) -> LeftBorder<'a, Message> {
    LeftBorder::new(content)
}

impl<'a, Message> LeftBorder<'a, Message> {
    fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let content = content.into();
        let size = content.as_widget().size_hint();

        LeftBorder {
            width: size.width.fluid(),
            height: size.height.fluid(),
            content,
            style_fn: StyleFn::default(),
        }
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn style(mut self, style_fn: impl Fn(&Theme) -> Border + 'a) -> Self {
        self.style_fn = StyleFn(Rc::new(style_fn));
        self
    }
}

impl<'a, Message> From<LeftBorder<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(container: LeftBorder<'a, Message>) -> Element<'a, Message> {
        Element::new(container)
    }
}

impl<'a, Message> Widget<Message, Theme, Renderer> for LeftBorder<'a, Message>
where
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::contained(&limits, self.width, self.height, |limits| {
            self.content.as_widget_mut().layout(tree, renderer, limits)
        })
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                tree,
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            tree,
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
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            tree,
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) where
        Renderer: iced::advanced::Renderer,
    {
        let bounds = layout.bounds();

        let border = self.style_fn.0(theme);
        draw_border(renderer, &border, bounds);

        if let Some(clipped_viewport) = bounds.intersection(viewport) {
            self.content.as_widget().draw(
                tree,
                renderer,
                theme,
                &renderer::Style {
                    text_color: renderer_style.text_color,
                },
                layout.children().next().unwrap(),
                cursor,
                &clipped_viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            tree,
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

fn draw_border<Renderer>(renderer: &mut Renderer, border: &Border, bounds: Rectangle)
where
    Renderer: iced::advanced::Renderer,
{
    if border.width > 0.0 {
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    width: border.width * 2.0,
                    ..bounds
                },
                border: border.clone(),
                shadow: Shadow::default(),
                snap: false,
            },
            border.color,
        )
    }
}

#[derive(Clone)]
struct StyleFn<'a>(Rc<dyn Fn(&iced::Theme) -> Border + 'a>);

impl<'a> Default for StyleFn<'a> {
    fn default() -> Self {
        StyleFn(Rc::new(|_theme: &iced::Theme| Border::default()))
    }
}
