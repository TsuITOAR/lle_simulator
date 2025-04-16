use std::time::{Duration, Instant};

use anyhow::Result;
use iced::widget::{button, column, container, row, text, Column, Container, Row};
use iced::Task;
use iced::{Alignment, Element, Length};
use lle_simulator::*;

#[allow(unused)]
use log::{debug, error, info, log_enabled, warn, Level};

mod gui;
use gui::*;

fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    // 使用新的应用程序构建API
    let app = iced::application(
        LleSimulator::title,
        LleSimulator::update,
        LleSimulator::view,
    );
    app.run()?;

    Ok(())
}

struct LleSimulator {
    simulator: Worker,
    draw1: DrawData,
    draw2: DrawData,
    panel: [Control; 6],
    pause: bool,
    last_update: Option<Instant>,
}

impl Default for LleSimulator {
    fn default() -> Self {
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
                RecordStep(_) => Control::new(|x| RecordStep(x as u32), "Record Step", None),
                SimuStep(_) => Control::new(SimuStep, "Simulation Step", None),
                Couple(v) => Control::new(Couple, "Couple Coefficient", v.into()),
            }
        };
        let simulator = Worker::new();
        Self {
            draw1: DrawData::new(simulator.get_state().0.len(), (WIDTH, HEIGHT)),
            draw2: DrawData::new(simulator.get_state().1.len(), (WIDTH, HEIGHT)),
            simulator,
            panel: array_init::from_iter(
                IntoIterator::into_iter(from_property_array(proper)).map(|x| init_from_property(x)),
            )
            .expect("initializing state"),
            pause: true,
            last_update: None,
        }
    }
}

impl LleSimulator {
    fn title(&self) -> String {
        "Lle Simulator".into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        log::info!("update message: {:?}", message);
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
                let state = self.simulator.get_state();
                self.draw1.push(state.0.to_owned());
                self.draw1.update().expect("refreshing status 1");
                self.draw2.push(state.1.to_owned());
                self.draw2.update().expect("refreshing status 2");
                if !self.pause {
                    const FPS: u64 = 60;
                    let duration: Duration = Duration::from_secs_f32(1. / FPS as f32);
                    let now = Instant::now();
                    return self
                        .last_update
                        .replace(now)
                        .map(|last| last + duration - now)
                        .map_or(Task::perform(async {}, |_| Message::Tick), |x| {
                            Task::perform(
                                async move {
                                    tokio::time::sleep(x).await;
                                },
                                |_| Message::Tick,
                            )
                        });
                }
            }
            Message::Pause => {
                self.pause = !self.pause;
                if !self.pause {
                    return Task::perform(async {}, |_| Message::Tick);
                }
            }
        };
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let mut control = column![]
            .spacing(20)
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill);

        let proper = self.simulator.get_property();
        for (c, w) in self
            .panel
            .iter()
            .zip(IntoIterator::into_iter(from_property_array(proper)))
        {
            control = control.push(c.view(w));
        }

        let pause_button = button(text(if self.pause { "Run" } else { "Pause" }))
            .on_press(Message::Pause)
            .padding(10);

        let tick_button = button(text("Step")).on_press(Message::Tick).padding(10);

        control = control.push(
            row![
                container(pause_button).padding(5),
                container(tick_button).padding(5)
            ]
            .align_y(Alignment::Center)
            .width(Length::Shrink),
        );

        container(control)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(5)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }
}
