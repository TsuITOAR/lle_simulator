use std::time::Duration;

use anyhow::Result;
use iced::{
    button, executor, Align, Application, Button, Clipboard, Column, Command, Container, Length,
    Settings, Subscription,
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
    draw: MyChart,
    panel: [Control; 5],
    pause: bool,
    button: button::State,
}

impl Application for LleSimulator {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        use WorkerUpdate::*;
        let simu = Worker::new();
        let proper = simu.get_property();
        let init_from_property = |p: WorkerUpdate| -> Control<f64> {
            match p {
                Alpha(v) => Control::new(Alpha, "Alpha", v.into()),
                Pump(v) => Control::new(Pump, "Pump", v.into()),
                Linear(v) => Control::new(Linear, "Linear", v.into()),
                RecordStep(_) => Control::new(|x| RecordStep(x as u64), "Record Step", None),
                SimuStep(_) => Control::new(SimuStep, "Simulation Step", None),
            }
        };
        (
            Self {
                simulator: Worker::new(),
                draw: Default::default(),
                panel: array_init::from_iter(
                    IntoIterator::into_iter(from_property_array(proper))
                        .map(|x| init_from_property(x)),
                )
                .expect("initializing state"),
                pause: true,
                button: button::State::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Lle Simulator".into()
    }
    fn subscription(&self) -> Subscription<Self::Message> {
        const FPS: u64 = 5;
        if !self.pause {
            iced::time::every(Duration::from_millis(1000 / FPS)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
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
                self.draw
                    .push(self.simulator.get_state().iter().map(|x| x.re).collect());
                self.draw.update().expect("refreshing status");
            }
            Message::Pause => self.pause = !self.pause,
        };
        Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        let mut control = Column::new()
            .spacing(20)
            .align_items(Align::Start)
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
            Button::new(
                &mut self.button,
                iced::Text::new(if self.pause { "Run" } else { "Pause" }),
            )
            .on_press(Message::Pause)
            .padding(10),
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
