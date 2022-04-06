use std::time::Duration;

use anyhow::Result;
use iced::{
    executor, Align, Application, Clipboard, Column, Command, Container, Length, Row, Subscription,
};
use lle_simulator::*;

#[allow(unused)]
use log::{debug, error, info, log_enabled, warn, Level};

mod gui;
use gui::*;

fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    Ok(())
}

struct LleSimulator {
    simulator: Worker,
    draw: MyChart,
    panel: [Control; 5],
}

fn map_property_to_idx(p: WorkerUpdate) -> usize {
    match p {
        WorkerUpdate::Alpha(_) => 0,
        WorkerUpdate::Pump(_) => 1,
        WorkerUpdate::Linear(_) => 2,
        WorkerUpdate::RecordStep(_) => 3,
        WorkerUpdate::SimuStep(_) => 4,
    }
}
fn map_idx_to_property(idx: usize, v: f64) -> WorkerUpdate {
    match idx {
        0 => WorkerUpdate::Alpha(v),
        1 => WorkerUpdate::Pump(v),
        2 => WorkerUpdate::Linear(v),
        3 => WorkerUpdate::RecordStep(v as u64),
        4 => WorkerUpdate::SimuStep(v),
        _ => unreachable!(),
    }
}
fn extract_property_value(p: WorkerUpdate) -> f64 {
    match p {
        WorkerUpdate::Alpha(v) => v,
        WorkerUpdate::Pump(v) => v,
        WorkerUpdate::Linear(v) => v,
        WorkerUpdate::RecordStep(_) => unreachable!(),
        WorkerUpdate::SimuStep(_) => unreachable!(),
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
                    Control::new(Alpha, "Alpha", proper.alpha.into()),
                    Control::new(Pump, "Pump", proper.pump.into()),
                    Control::new(Linear, "Linear", proper.linear.into()),
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
        const FPS: u64 = 60;
        iced::time::every(Duration::from_millis(1000 / FPS)).map(|_| Message::Tick)
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        match message {
            Message::Input(v) => {
                v.apply_or_warn(|v| self.simulator.set_property(v));
            }
            Message::Slide((v, t)) => match t {
                SlideMessage::SetMax => {
                    v.apply_or_warn(|x| {
                        self.panel[map_property_to_idx(x)]
                            .range
                            .as_mut()
                            .map_or_else(
                                || warn!("none slider panel returned SetMax message"),
                                |r| r.higher = extract_property_value(x),
                            )
                    });
                }
                SlideMessage::SetMin => {
                    v.apply_or_warn(|x| {
                        self.panel[map_property_to_idx(x)]
                            .range
                            .as_mut()
                            .map_or_else(
                                || warn!("none slider panel returned SetMax message"),
                                |r| r.lower = extract_property_value(x),
                            )
                    });
                }
                SlideMessage::SetVal => {
                    v.apply_or_warn(|v| self.simulator.set_property(v));
                }
            },
            Message::Tick => self.simulator.tick(),
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

        let mut control = Column::new()
            .spacing(20)
            .align_items(Align::Start)
            .width(Length::Fill)
            .height(Length::Fill);
        let proper = self.simulator.get_property();
        self.panel
            .iter()
            .enumerate()
            .for_each(|(idx, x)| control = control.push(x.view()));

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
