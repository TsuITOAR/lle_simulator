use std::{cell::RefCell, time::Duration};

use anyhow::Result;
use iced::{
    executor, Align, Application, Clipboard, Column, Command, Container, Element,
    HorizontalAlignment, Length, Row, Subscription, Text, TextInput,
};
use jkplot::*;
use lle_simulator::*;
use plotters::{coord::Shift, prelude::*};
use plotters_iced::{Chart, ChartWidget};

mod style {
    use iced::Color;

    pub struct ChartContainer;
    impl iced::container::StyleSheet for ChartContainer {
        fn style(&self) -> iced::container::Style {
            iced::container::Style {
                background: Some(Color::BLACK.into()),
                text_color: Some(Color::WHITE),
                ..Default::default()
            }
        }
    }
}

fn main() -> Result<()> {
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum SlideMessage {
    SetMax,
    SetMin,
    SetVal,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Input(WorkerUpdate),
    Slide((WorkerUpdate, SlideMessage)),
    Tick,
}

struct LleSimulator {
    simulator: Worker,
    draw: MyChart,
    panel: [Control; 5],
}

struct Control<T = f64> {
    call_back: fn(T) -> WorkerUpdate,
    desc: String,
    range: Option<(f64, f64)>,
}

impl<T> Control<T> {
    fn new(
        call_back: fn(T) -> WorkerUpdate,
        desc: impl Into<String>,
        range: Option<(f64, f64)>,
    ) -> Self {
        Self {
            call_back,
            desc: desc.into(),
            range,
        }
    }
    fn view(&self, value: T) -> Element<Message> {
        let desc = Row::new()
            .spacing(5)
            .align_items(Align::Start)
            .width(Length::Fill)
            .push(
                Text::new(&self.desc)
                    .width(Length::FillPortion(4))
                    .horizontal_alignment(HorizontalAlignment::Right),
            ).push(
                TextInput::new()
            );
        let mut c = Column::new()
            .spacing(20)
            .align_items(Align::Start)
            .width(Length::Fill)
            .push(desc);
        /* if let Some((l, h)) = self.range {
            c = c.push(Row::new().push(TextInput::new(state, placeholder, value, on_change)))
        } */
        c.into()
    }
}

#[derive(Default)]
struct MyChart {
    data: Vec<Vec<f64>>,
    plot: RefCell<RawAnimator>,
    map: RawMapVisualizer,
}

impl Chart<Message> for MyChart {
    fn build_chart<DB: DrawingBackend>(&self, builder: ChartBuilder<DB>) {
        unreachable!()
    }
    fn draw_chart<DB: DrawingBackend>(&self, root: DrawingArea<DB, Shift>) {
        let (upper, lower) = root.split_horizontally(50.percent());
        RefCell::borrow_mut(&self.plot)
            .new_frame_on(
                self.data
                    .last()
                    .expect("drawing line of last raw")
                    .iter()
                    .enumerate()
                    .map(|(x, y)| (x as f64, *y)),
                &upper,
            )
            .unwrap();
        self.map.draw_on(&self.data, &lower).unwrap();
    }
}

impl MyChart {
    fn view(&mut self) -> Element<Message> {
        Container::new(
            Column::new()
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(5)
                .push(ChartWidget::new(self).height(Length::Fill)),
        )
        .style(style::ChartContainer)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Align::Center)
        .align_y(Align::Center)
        .into()
    }
}

impl Application for LleSimulator {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        use WorkerUpdate::*;
        let simu = Worker::new();
        let proper = simu.get_property();
        (
            Self {
                simulator: Worker::new(),
                draw: Default::default(),
                panel: [
                    Control::new(
                        Alpha,
                        "Alpha",
                        (proper.alpha - 5., proper.alpha + 5.).into(),
                    ),
                    Control::new(Pump, "Pump", (proper.pump - 5., proper.pump + 5.).into()),
                    Control::new(
                        Linear,
                        "Linear",
                        (proper.linear - 5., proper.linear + 5.).into(),
                    ),
                    Control::new(|x| RecordStep(x as u64), "Record Step", None),
                    Control::new(SimuStep, "Simulation Step", None),
                ],
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Lle Simulator".into()
    }
    fn subscription(&self) -> Subscription<Self::Message> {
        const FPS: u64 = 50;
        iced::time::every(Duration::from_millis(1000 / FPS)).map(|_| Message::Tick)
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        match message {
            Message::Input(v) => self.simulator.set_property(v),
            Message::Slide((v, t)) => match t {
                SlideMessage::SetMax => todo!(),
                SlideMessage::SetMin => todo!(),
                SlideMessage::SetVal => todo!(),
            },
            Message::Tick => todo!(),
        };
        Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        let plot = Column::new()
            .spacing(20)
            .align_items(Align::Start)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(self.draw.view());

        let control = Column::new()
            .spacing(20)
            .align_items(Align::Start)
            .width(Length::Fill)
            .height(Length::Fill);

        let content = Row::new()
            .spacing(20)
            .align_items(Align::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(plot)
            .push(control);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(5)
            .center_x()
            .center_y()
            .into()
    }
}
