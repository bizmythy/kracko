//! Layout state of a [`HorizontalScroll`] container.
//!
//! [`HorizontalScroll`]: super::HorizontalScroll

use iced::widget::Id;
use iced::widget::operation;
use iced::widget::scrollable::AbsoluteOffset;
use iced::{Size, Task};

/// The default width of a column, as a fraction of the viewport height.
///
/// A freshly spawned column occupies an 8:9 (width:height) region.
pub const DEFAULT_RATIO: f32 = 8.0 / 9.0;

/// The narrowest a user may resize a column to, as a fraction of the
/// viewport height.
const MIN_RATIO: f32 = 0.25;

/// The widest a user may resize a column to, as a fraction of the
/// viewport height.
const MAX_RATIO: f32 = 4.0;

/// A unique identifier of a column in a [`State`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Pane(usize);

/// An interaction produced by a [`HorizontalScroll`] widget.
///
/// Feed these back into [`State::update`].
///
/// [`HorizontalScroll`]: super::HorizontalScroll
#[derive(Debug, Clone, Copy)]
pub enum Event {
    /// A pane was clicked.
    Clicked(Pane),
    /// A divider was dragged. `delta` is the width change of the pane to
    /// its left, as a fraction of the viewport height.
    Resized { pane: Pane, delta: f32 },
    /// The container was laid out with a new viewport size.
    ViewportResized(Size),
    /// The strip was scrolled.
    Scrolled(AbsoluteOffset),
}

/// A state change produced by [`State::update`] that the application may
/// want to react to.
#[derive(Debug, Clone, Copy)]
pub enum Action {
    /// Focus moved to the given pane.
    FocusChanged(Pane),
}

struct Column<T> {
    pane: Pane,
    content: T,
    /// Width of the column as a fraction of the viewport height.
    ratio: f32,
}

/// The ordered columns of a [`HorizontalScroll`], plus focus, sizing, and
/// scroll bookkeeping.
///
/// [`HorizontalScroll`]: super::HorizontalScroll
pub struct State<T> {
    columns: Vec<Column<T>>,
    focus: Option<Pane>,
    next_id: usize,
    spacing: f32,
    scrollable_id: Id,
    viewport: Option<Size>,
    scroll_x: f32,
}

impl<T> State<T> {
    /// Creates a new [`State`] holding a single pane with the given content.
    pub fn new(content: T) -> (Self, Pane) {
        let pane = Pane(0);

        (
            Self {
                columns: vec![Column {
                    pane,
                    content,
                    ratio: DEFAULT_RATIO,
                }],
                focus: Some(pane),
                next_id: 1,
                spacing: 10.0,
                scrollable_id: Id::unique(),
                viewport: None,
                scroll_x: 0.0,
            },
            pane,
        )
    }

    /// Sets the width of the gap (and resize handle) between columns.
    pub fn set_spacing(&mut self, spacing: f32) {
        self.spacing = spacing;
    }

    pub fn spacing(&self) -> f32 {
        self.spacing
    }

    /// The [`Id`] of the underlying scrollable.
    pub fn id(&self) -> Id {
        self.scrollable_id.clone()
    }

    pub fn len(&self) -> usize {
        self.columns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    pub fn focused(&self) -> Option<Pane> {
        self.focus
    }

    /// Focuses the given pane, if it exists.
    pub fn focus(&mut self, pane: Pane) {
        if self.index_of(pane).is_some() {
            self.focus = Some(pane);
        }
    }

    pub fn get(&self, pane: Pane) -> Option<&T> {
        self.columns
            .iter()
            .find(|column| column.pane == pane)
            .map(|column| &column.content)
    }

    pub fn get_mut(&mut self, pane: Pane) -> Option<&mut T> {
        self.columns
            .iter_mut()
            .find(|column| column.pane == pane)
            .map(|column| &mut column.content)
    }

    /// Iterates over the panes and their contents, in left-to-right order.
    pub fn iter(&self) -> impl Iterator<Item = (Pane, &T)> {
        self.columns
            .iter()
            .map(|column| (column.pane, &column.content))
    }

    /// Iterates over `(pane, content, ratio)` in left-to-right order.
    pub(super) fn columns(&self) -> impl Iterator<Item = (Pane, &T, f32)> {
        self.columns
            .iter()
            .map(|column| (column.pane, &column.content, column.ratio))
    }

    /// Inserts a new pane immediately to the right of `pane` and returns its
    /// id. Appends at the far right if `pane` no longer exists.
    pub fn insert_right_of(&mut self, pane: Pane, content: T) -> Pane {
        let index = self
            .index_of(pane)
            .map(|index| index + 1)
            .unwrap_or(self.columns.len());

        self.insert_at(index, content)
    }

    /// Inserts a new pane immediately to the left of `pane` and returns its
    /// id. Appends at the far right if `pane` no longer exists.
    pub fn insert_left_of(&mut self, pane: Pane, content: T) -> Pane {
        let index = self.index_of(pane).unwrap_or(self.columns.len());

        self.insert_at(index, content)
    }

    /// Appends a new pane at the far right and returns its id.
    pub fn push(&mut self, content: T) -> Pane {
        self.insert_at(self.columns.len(), content)
    }

    /// Closes the given pane, returning its content and the neighboring
    /// pane that inherits focus (`None` if it was the last pane).
    pub fn close(&mut self, pane: Pane) -> Option<(T, Option<Pane>)> {
        let index = self.index_of(pane)?;
        let column = self.columns.remove(index);

        let neighbor = self
            .columns
            .get(index)
            .or_else(|| self.columns.last())
            .map(|column| column.pane);

        if self.focus == Some(pane) {
            self.focus = neighbor;
        }

        Some((column.content, neighbor))
    }

    /// Processes an [`Event`] produced by the [`HorizontalScroll`] widget.
    ///
    /// [`HorizontalScroll`]: super::HorizontalScroll
    pub fn update(&mut self, event: Event) -> Option<Action> {
        match event {
            Event::Clicked(pane) => {
                let changed = self.focus != Some(pane);
                self.focus(pane);

                changed.then_some(Action::FocusChanged(pane))
            },
            Event::Resized { pane, delta } => {
                if let Some(column) =
                    self.columns.iter_mut().find(|column| column.pane == pane)
                {
                    column.ratio =
                        (column.ratio + delta).clamp(MIN_RATIO, MAX_RATIO);
                }

                None
            },
            Event::ViewportResized(size) => {
                self.viewport = Some(size);

                None
            },
            Event::Scrolled(offset) => {
                self.scroll_x = offset.x;

                None
            },
        }
    }

    /// Produces a [`Task`] that scrolls the strip just enough to bring the
    /// given pane fully into view, à la Niri.
    ///
    /// Does nothing if the pane is already fully visible or the viewport
    /// size is not known yet.
    pub fn snap_to<Message>(&mut self, pane: Pane) -> Task<Message>
    where
        Message: Send + 'static,
    {
        let Some(viewport) = self.viewport else {
            return Task::none();
        };

        let Some(index) = self.index_of(pane) else {
            return Task::none();
        };

        let height = viewport.height.max(1.0);
        let start: f32 = self.columns[..index]
            .iter()
            .map(|column| column.ratio * height + self.spacing)
            .sum();
        let end = start + self.columns[index].ratio * height;

        let visible_start = self.scroll_x;
        let visible_end = self.scroll_x + viewport.width;

        let target = if start < visible_start {
            Some((start - self.spacing).max(0.0))
        } else if end > visible_end {
            Some(end + self.spacing - viewport.width)
        } else {
            None
        };

        match target {
            Some(x) => {
                self.scroll_x = x;

                operation::scroll_to(
                    self.scrollable_id.clone(),
                    AbsoluteOffset { x, y: 0.0 },
                )
            },
            None => Task::none(),
        }
    }

    fn index_of(&self, pane: Pane) -> Option<usize> {
        self.columns.iter().position(|column| column.pane == pane)
    }

    fn insert_at(&mut self, index: usize, content: T) -> Pane {
        let pane = Pane(self.next_id);
        self.next_id += 1;

        self.columns.insert(
            index,
            Column {
                pane,
                content,
                ratio: DEFAULT_RATIO,
            },
        );

        pane
    }
}
