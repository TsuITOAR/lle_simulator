use std::time::{Duration, Instant};

use anyhow::Result;
use iced::{
    button, executor, Align, Application, Button, Clipboard, Column, Command, Container, Length,
    Row, Settings,
};
use lle_simulator::*;

#[allow(unused)]
use log::{debug, error, info, log_enabled, warn, Level};

mod gui;
use gui::*;

fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn)
        .init();
    LleSimulator::run(Settings::default())?;
    Ok(())
}

struct LleSimulator {
    simulator: Worker,
    draw: DrawData,
    panel: [Control; 5],
    pause: bool,
    pause_button: button::State,
    tick_button: button::State,
    last_update: Option<Instant>,
}

impl Application for LleSimulator {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        const WIDTH: usize = 640;
        const HEIGHT: usize = 640;
        use WorkerUpdate::*;
        let simulation = Worker::new();
        let proper = simulation.get_property();
        let init_from_property = |p: WorkerUpdate| -> Control<f64> {
            match p {
                Alpha(v) => Control::new(Alpha, "Alpha", v.into()),
                Pump(v) => Control::new(Pump, "Pump", v.into()),
                Linear(v) => Control::new(Linear, "Linear", v.into()),
                RecordStep(_) => Control::new(|x| RecordStep(x as u64), "Record Step", None),
                SimuStep(_) => Control::new(SimuStep, "Simulation Step", None),
            }
        };
        let simulator = Worker::new();
        (
            Self {
                draw: DrawData::new(simulator.get_state().len(), (WIDTH, HEIGHT)),
                simulator,
                panel: array_init::from_iter(
                    IntoIterator::into_iter(from_property_array(proper))
                        .map(|x| init_from_property(x)),
                )
                .expect("initializing state"),
                pause: true,
                pause_button: button::State::new(),
                tick_button: button::State::new(),
                last_update: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Lle Simulator".into()
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
                        self.panel[map_property_to_idx(x)].range_mut().map_or_else(
                            || warn!("none slider panel returned SetMax message"),
                            |r| r.higher = extract_property_value(x),
                        )
                    });
                }
                SlideMessage::SetMin => {
                    v.apply_or_warn(|x| {
                        self.panel[map_property_to_idx(x)].range_mut().map_or_else(
                            || warn!("none slider panel returned SetMin message"),
                            |r| r.lower = extract_property_value(x),
                        )
                    });
                }
                SlideMessage::SetVal => {
                    v.apply_or_warn(|v| self.simulator.set_property(v));
                }
            },
            Message::Tick => {
                self.simulator.tick();
                self.draw.push(self.simulator.get_state().to_owned());
                self.draw.update().expect("refreshing status");
                if !self.pause {
                    const FPS: u64 = 60;
                    let duration: Duration = Duration::from_secs_f32(1. / FPS as f32);
                    let now = Instant::now();
                    return self
                        .last_update
                        .replace(now)
                        .map(|last| last + duration - now)
                        .map_or(async { Message::Tick }.into(), |x| {
                            async move {
                                tokio::time::sleep(x).await;
                                Message::Tick
                            }
                            .into()
                        });
                }
            }
            Message::Pause => {
                self.pause = !self.pause;
                if !self.pause {
                    return Command::from(async { Message::Tick });
                }
            }
        };
        Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        let mut control = Column::new()
            .spacing(20)
            .align_items(Align::Center)
            .width(Length::Fill)
            .height(Length::Fill);
        let proper = self.simulator.get_property();
        for (c, w) in self
            .panel
            .iter_mut()
            .zip(IntoIterator::into_iter(from_property_array(proper)))
        {
            control = control.push(c.view(w));
        }
        control = control.push(
            Row::new()
                .align_items(Align::Center)
                .width(Length::Shrink)
                .push(
                    Container::new(
                        Button::new(
                            &mut self.pause_button,
                            iced::Text::new(if self.pause { "Run" } else { "Pause" }),
                        )
                        .on_press(Message::Pause)
                        .padding(10),
                    )
                    .padding(5),
                )
                .push(
                    Container::new(
                        Button::new(&mut self.tick_button, iced::Text::new("Step"))
                            .on_press(Message::Tick)
                            .padding(10),
                    )
                    .padding(5),
                ),
        );

        Container::new(control)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(5)
            .center_x()
            .center_y()
            .into()
    }
}
