use iced::widget::{button, column, container, text};
use iced::{Element, Result, Task};

pub fn main() -> Result {
    iced::application(Counter::new, Counter::update, Counter::view)
        .title("Icy Counter - Iced")
        .run()
}

struct Counter {
    value: i32,
}

impl Counter {
    fn new() -> (Self, Task<Message>) {
        (Self { value: 0 }, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Increment => self.value += 1,
            Message::Decrement => self.value -= 1,
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let content = column![
            button("+").on_press(Message::Increment),
            text(self.value).size(50),
            button("-").on_press(Message::Decrement),
        ]
        .spacing(20)
        .padding(20);

        container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fill)
            .into()
    }
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
}
