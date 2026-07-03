//! A draggable vertical divider between two columns.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::{
    Element, Event, Length, Rectangle, Size, Theme, border, mouse,
};

/// A vertical drag handle that reports horizontal drag deltas, in pixels,
/// through its `on_drag` callback.
pub struct Divider<'a, Message> {
    width: f32,
    on_drag: Box<dyn Fn(f32) -> Message + 'a>,
}

impl<'a, Message> Divider<'a, Message> {
    pub fn new(width: f32, on_drag: impl Fn(f32) -> Message + 'a) -> Self {
        Self {
            width,
            on_drag: Box::new(on_drag),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct DragState {
    is_dragging: bool,
    last_x: f32,
}

impl<Message, Renderer> Widget<Message, Theme, Renderer>
    for Divider<'_, Message>
where
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<DragState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(DragState::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fixed(self.width),
            height: Length::Fill,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, Length::Fixed(self.width), Length::Fill)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<DragState>();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) =
                    cursor.position_over(layout.bounds())
                {
                    state.is_dragging = true;
                    state.last_x = position.x;
                    shell.capture_event();
                }
            },
            Event::Mouse(mouse::Event::CursorMoved { .. })
                if state.is_dragging =>
            {
                if let Some(position) = cursor.position() {
                    let delta = position.x - state.last_x;

                    if delta != 0.0 {
                        state.last_x = position.x;
                        shell.publish((self.on_drag)(delta));
                    }

                    shell.capture_event();
                }
            },
            Event::Mouse(mouse::Event::ButtonReleased(
                mouse::Button::Left,
            )) if state.is_dragging => {
                state.is_dragging = false;
                shell.capture_event();
            },
            _ => {},
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<DragState>();

        if state.is_dragging || cursor.is_over(layout.bounds()) {
            mouse::Interaction::ResizingHorizontally
        } else {
            mouse::Interaction::None
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<DragState>();
        let palette = theme.extended_palette();

        let is_active = state.is_dragging || cursor.is_over(bounds);

        let (line_width, color) = if is_active {
            (3.0, palette.primary.strong.color)
        } else {
            (1.0, palette.background.strong.color)
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x + (bounds.width - line_width) / 2.0,
                    y: bounds.y,
                    width: line_width,
                    height: bounds.height,
                },
                border: border::rounded(line_width / 2.0),
                ..renderer::Quad::default()
            },
            color,
        );
    }
}

impl<'a, Message, Renderer> From<Divider<'a, Message>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: 'a + renderer::Renderer,
{
    fn from(divider: Divider<'a, Message>) -> Self {
        Element::new(divider)
    }
}
