//! The view side of the container: assembles columns, dividers, the
//! horizontal scrollable, and the size sensor into a single element.

use std::rc::Rc;

use iced::widget::{Row, container, responsive, scrollable, sensor};
use iced::{Element, Length};

use super::divider::Divider;
use super::focus_area::FocusArea;
use super::state::{Event, Pane, State};

/// A Niri-style horizontally scrolling strip of full-height columns.
///
/// See the [module docs](super) for an overview.
pub struct HorizontalScroll<'a, T, Message> {
    state: &'a State<T>,
    on_event: Rc<dyn Fn(Event) -> Message + 'a>,
    view: Box<dyn Fn(Pane, &'a T, bool) -> Element<'a, Message> + 'a>,
}

impl<'a, T, Message> HorizontalScroll<'a, T, Message>
where
    Message: Clone + 'a,
{
    /// Creates a [`HorizontalScroll`] over the given [`State`].
    ///
    /// `on_event` maps container interactions to your message type; feed
    /// them back into [`State::update`]. `view` renders the content of each
    /// pane, receiving its id, content, and whether it is focused.
    pub fn new(
        state: &'a State<T>,
        on_event: impl Fn(Event) -> Message + 'a,
        view: impl Fn(Pane, &'a T, bool) -> Element<'a, Message> + 'a,
    ) -> Self {
        Self {
            state,
            on_event: Rc::new(on_event),
            view: Box::new(view),
        }
    }
}

impl<'a, T, Message> From<HorizontalScroll<'a, T, Message>>
    for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(widget: HorizontalScroll<'a, T, Message>) -> Self {
        let HorizontalScroll {
            state,
            on_event,
            view,
        } = widget;

        responsive(move |size| {
            let height = size.height.max(1.0);
            let focused = state.focused();

            let mut children: Vec<Element<'a, Message>> = Vec::new();
            let mut previous: Option<Pane> = None;

            for (pane, content, ratio) in state.columns() {
                // The divider between two columns resizes the one to its
                // left, converting pixel drags into height-relative deltas.
                if let Some(left) = previous {
                    let on_event = Rc::clone(&on_event);

                    children.push(
                        Divider::new(state.spacing(), move |delta| {
                            on_event(Event::Resized {
                                pane: left,
                                delta: delta / height,
                            })
                        })
                        .into(),
                    );
                }

                let body = container(view(pane, content, focused == Some(pane)))
                    .width(ratio * height)
                    .height(Length::Fill);

                children.push(
                    FocusArea::new(body, on_event(Event::Clicked(pane)))
                        .into(),
                );

                previous = Some(pane);
            }

            let strip = scrollable(
                Row::with_children(children).height(Length::Fill),
            )
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::new(),
            ))
            .id(state.id())
            .width(Length::Fill)
            .height(Length::Fill)
            .on_scroll({
                let on_event = Rc::clone(&on_event);
                move |viewport| {
                    on_event(Event::Scrolled(viewport.absolute_offset()))
                }
            });

            // The sensor keeps `State` informed of the viewport size, which
            // it needs to translate column ratios into scroll offsets.
            sensor(strip)
                .on_show({
                    let on_event = Rc::clone(&on_event);
                    move |size| on_event(Event::ViewportResized(size))
                })
                .on_resize({
                    let on_event = Rc::clone(&on_event);
                    move |size| on_event(Event::ViewportResized(size))
                })
                .into()
        })
        .into()
    }
}
