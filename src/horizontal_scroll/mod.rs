//! A Niri-style horizontally scrolling tiling container.
//!
//! Unlike [`iced::widget::pane_grid`], which tiles panes both vertically and
//! horizontally inside a fixed region, this container arranges its panes as a
//! single row of full-height columns on an infinite horizontal strip. New
//! panes are only ever inserted to the left or right of an existing pane, and
//! the whole strip scrolls horizontally when it outgrows the viewport.
//!
//! Each column occupies a width expressed as a fraction of the viewport
//! height (8:9 by default, see [`DEFAULT_RATIO`]), and can be resized by
//! dragging the divider on its right edge.
//!
//! Typical wiring:
//! - keep a [`State`] in your application state,
//! - build a [`HorizontalScroll`] in `view`, forwarding its [`Event`]s to a
//!   message of your own,
//! - feed those events back into [`State::update`] and react to the returned
//!   [`Action`]s.

mod divider;
mod focus_area;
mod state;
mod widget;

#[allow(unused_imports)]
pub use state::{Action, DEFAULT_RATIO, Event, Pane, State};
pub use widget::HorizontalScroll;
