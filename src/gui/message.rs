use std::ops::RangeInclusive;

use super::*;

use iced::widget::{slider, text_input, Text};
use lle_simulator::WorkerUpdate;

#[derive(Debug, Clone, Copy)]
pub enum SlideMessage {
    SetMax,
    SetMin,
    SetVal,
}

#[derive(Debug, Clone)]
pub enum Message {
    Input(NewValue),
    Slide((NewValue, SlideMessage)),
    Tick,
    Pause,
}

#[derive(Debug, Clone)]
pub enum NewValue {
    Number(WorkerUpdate),
    Nan(String),
}

impl NewValue {
    pub fn apply_or_warn<T>(self, f: impl FnOnce(WorkerUpdate) -> T) -> Option<T> {
        match self {
            NewValue::Nan(s) => {
                warn!("illegal input {}", s);
                None
            }
            NewValue::Number(v) => Some(f(v)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Control<T = f64> {
    call_back: fn(T) -> WorkerUpdate,
    desc: String,
    range: Option<Range<T>>,
}

#[derive(Debug, Clone, Default)]
pub struct Range<T> {
    pub lower: T,
    pub higher: T,
}

impl Range<f64> {
    const RANGE_LEN: f64 = 10.;
    pub fn from_center(center: f64) -> Self {
        Self {
            lower: center - Self::RANGE_LEN / 2.,
            higher: center + Self::RANGE_LEN / 2.,
            ..Default::default()
        }
    }
}

impl Control<f64> {
    pub fn new(
        call_back: fn(f64) -> WorkerUpdate,
        desc: impl Into<String>,
        center: Option<f64>,
    ) -> Self {
        Self {
            call_back,
            desc: desc.into(),
            range: center.map(Range::from_center),
        }
    }
    pub fn range_mut(&mut self) -> Option<&mut Range<f64>> {
        self.range.as_mut()
    }
    pub fn view(&self, value: WorkerUpdate) -> Element<Message> {
        const INPUT_WIDTH_PORTION: u16 = 8;
        let v = property_value_to_string(value);
        let call_back = self.call_back;
        let desc = Row::new()
            .spacing(5)
            .align_y(Alignment::Start)
            .width(Length::Fill)
            .push(
                Text::new(&self.desc)
                    .width(Length::FillPortion(INPUT_WIDTH_PORTION))
                    .align_x(Alignment::End),
            )
            .push(
                text_input("current value", &v)
                    .on_input(move |x| {
                        x.parse().map_or(Message::Input(NewValue::Nan(x)), |x| {
                            Message::Input(NewValue::Number(call_back(x)))
                        })
                    })
                    .width(Length::FillPortion(10 - INPUT_WIDTH_PORTION)),
            );
        let mut c = Column::new()
            .spacing(20)
            .align_x(Alignment::Start)
            .width(Length::Fill)
            .push(desc);
        if let Some(Range { lower, higher }) = self.range {
            let call_back_builder = |x: SlideMessage| {
                move |new: String| {
                    new.parse().map_or(Message::Input(NewValue::Nan(new)), |n| {
                        Message::Slide((NewValue::Number(call_back(n)), x))
                    })
                }
            };
            c = c.push(
                Row::new()
                    .align_y(Alignment::Center)
                    .push(
                        text_input("lower bound", &lower.to_string())
                            .on_input(call_back_builder(SlideMessage::SetMin)),
                    )
                    .push(
                        Container::new(
                            slider(
                                RangeInclusive::new(lower, higher),
                                extract_property_value(value),
                                move |new: f64| {
                                    Message::Slide((
                                        NewValue::Number(call_back(new)),
                                        SlideMessage::SetVal,
                                    ))
                                },
                            )
                            .width(Length::Fill)
                            .step((higher - lower) / 10000.),
                        )
                        .width(Length::FillPortion(INPUT_WIDTH_PORTION))
                        .padding(5),
                    )
                    .push(
                        text_input("higher bound", &higher.to_string())
                            .on_input(call_back_builder(SlideMessage::SetMax))
                            .width(Length::FillPortion(1)),
                    ),
            );
        }
        c.into()
    }
}
