//! The application: a strip of terminals inside a [`HorizontalScroll`].

use std::collections::HashMap;

use iced::widget::{button, column, container, responsive, row, text};
use iced::{Element, Length, Subscription, Task, alignment, window};
use iced_term::TerminalView;

use crate::horizontal_scroll::{self, HorizontalScroll, Pane};
use crate::style;
use crate::terminal;

pub struct App {
    /// Column layout; each pane holds the id of a terminal in `tabs`.
    layout: horizontal_scroll::State<u64>,
    tabs: HashMap<u64, iced_term::Terminal>,
    term_settings: iced_term::settings::Settings,
    next_terminal_id: u64,
}

#[derive(Debug, Clone)]
pub enum Event {
    OpenLeft(Pane),
    OpenRight(Pane),
    Close(Pane),
    Layout(horizontal_scroll::Event),
    Terminal(iced_term::Event),
}

impl App {
    pub fn new() -> (Self, Task<Event>) {
        let term_settings = terminal::settings();

        let initial_id = 0;
        let tab = terminal::spawn(initial_id, term_settings.clone());
        let focus = TerminalView::focus(tab.widget_id().clone());

        let mut tabs = HashMap::new();
        tabs.insert(initial_id, tab);

        let (layout, _) = horizontal_scroll::State::new(initial_id);

        (
            App {
                layout,
                tabs,
                term_settings,
                next_terminal_id: initial_id + 1,
            },
            focus,
        )
    }

    pub fn title(&self) -> String {
        String::from("Terminal strip")
    }

    pub fn update(&mut self, event: Event) -> Task<Event> {
        match event {
            Event::OpenLeft(pane) => self.open(pane, Side::Left),
            Event::OpenRight(pane) => self.open(pane, Side::Right),
            Event::Close(pane) => self.close(pane),
            Event::Layout(event) => match self.layout.update(event) {
                Some(horizontal_scroll::Action::FocusChanged(pane)) => {
                    self.focus(pane)
                },
                None => Task::none(),
            },
            Event::Terminal(iced_term::Event::BackendCall(id, cmd)) => {
                let action = match self.tabs.get_mut(&id) {
                    Some(tab) => {
                        tab.handle(iced_term::Command::ProxyToBackend(cmd))
                    },
                    None => return Task::none(),
                };

                if action == iced_term::actions::Action::Shutdown {
                    let pane = self
                        .layout
                        .iter()
                        .find(|(_, tab_id)| **tab_id == id)
                        .map(|(pane, _)| pane);

                    if let Some(pane) = pane {
                        return self.close(pane);
                    }
                }

                Task::none()
            },
        }
    }

    pub fn view(&'_ self) -> Element<'_, Event> {
        let strip = HorizontalScroll::new(
            &self.layout,
            Event::Layout,
            |pane, tab_id, is_focused| {
                self.pane_view(pane, *tab_id, is_focused)
            },
        );

        container(strip)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Event> {
        Subscription::batch(self.tabs.values().map(|tab| tab.subscription()))
            .map(Event::Terminal)
    }

    fn pane_view(
        &self,
        pane: Pane,
        tab_id: u64,
        is_focused: bool,
    ) -> Element<'_, Event> {
        let title_color = if is_focused {
            style::TITLE_COLOR_FOCUSED
        } else {
            style::TITLE_COLOR_UNFOCUSED
        };

        let title = row![
            "Term",
            text(tab_id.to_string()).color(title_color),
        ]
        .spacing(5);

        let control = |label, event| {
            button(
                text(label)
                    .align_x(alignment::Horizontal::Center)
                    .size(14),
            )
            .padding(3)
            .on_press(event)
        };

        let title_bar = container(
            row![
                title,
                iced::widget::space::horizontal(),
                control("+ Left", Event::OpenLeft(pane)),
                control("+ Right", Event::OpenRight(pane)),
                control("Close", Event::Close(pane))
                    .style(button::danger),
            ]
            .spacing(5)
            .align_y(alignment::Vertical::Center),
        )
        .padding(10)
        .width(Length::Fill)
        .style(if is_focused {
            style::title_bar_focused
        } else {
            style::title_bar_active
        });

        let tab = self
            .tabs
            .get(&tab_id)
            .expect("terminal with target id not found");

        let term = container(responsive(move |_| {
            TerminalView::show(tab).map(Event::Terminal)
        }))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(5);

        container(column![title_bar, term])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(if is_focused {
                style::pane_focused
            } else {
                style::pane_active
            })
            .into()
    }

    fn open(&mut self, pane: Pane, side: Side) -> Task<Event> {
        let id = self.next_terminal_id;
        self.next_terminal_id += 1;

        let tab = terminal::spawn(id, self.term_settings.clone());
        self.tabs.insert(id, tab);

        let new_pane = match side {
            Side::Left => self.layout.insert_left_of(pane, id),
            Side::Right => self.layout.insert_right_of(pane, id),
        };

        self.layout.focus(new_pane);
        self.focus(new_pane)
    }

    fn close(&mut self, pane: Pane) -> Task<Event> {
        let Some((tab_id, neighbor)) = self.layout.close(pane) else {
            return Task::none();
        };

        self.tabs.remove(&tab_id);

        match neighbor {
            Some(neighbor) => {
                self.layout.focus(neighbor);
                self.focus(neighbor)
            },
            None => window::latest().and_then(window::close),
        }
    }

    /// Focuses the terminal widget in `pane` and scrolls it into view.
    fn focus(&mut self, pane: Pane) -> Task<Event> {
        let focus = match self
            .layout
            .get(pane)
            .and_then(|tab_id| self.tabs.get(tab_id))
        {
            Some(tab) => TerminalView::focus(tab.widget_id().clone()),
            None => Task::none(),
        };

        Task::batch([focus, self.layout.snap_to(pane)])
    }
}

enum Side {
    Left,
    Right,
}
