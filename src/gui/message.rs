use std::{cell::RefCell, ops::RangeInclusive};

use iced::{
    slider, text_input, Align, Column, Container, Element, HorizontalAlignment, Length, Row,
    Slider, Text, TextInput,
};
use jkplot::{RawAnimator, RawMapVisualizer};
use lle_simulator::WorkerUpdate;
use plotters::{coord::Shift, style::AsRelative};
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingArea, DrawingBackend};

use super::*;

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
pub struct Control<T = f64> {
    call_back: fn(T) -> WorkerUpdate,
    desc: String,
    input: text_input::State,
    range: Option<Range<T>>,
}

#[derive(Debug, Clone, Default)]
pub struct Range<T> {
    pub lower: T,
    pub higher: T,
    input_lower: text_input::State,
    input_slide: slider::State,
    input_higher: text_input::State,
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
            input: text_input::State::new(),
            call_back,
            desc: desc.into(),
            range: center.map(Range::from_center),
        }
    }
    pub fn range_mut(&mut self) -> Option<&mut Range<f64>> {
        self.range.as_mut()
    }
    pub fn view(&mut self, value: WorkerUpdate) -> Element<Message> {
        const INPUT_WIDTH_PORTION: u16 = 8;
        let v = property_value_to_string(value);
        let call_back = self.call_back;
        let desc = Row::new()
            .spacing(5)
            .align_items(Align::Start)
            .width(Length::Fill)
            .push(
                Text::new(&self.desc)
                    .width(Length::FillPortion(INPUT_WIDTH_PORTION))
                    .horizontal_alignment(HorizontalAlignment::Right),
            )
            .push(
                TextInput::new(&mut self.input, "current value", &v, move |x| {
                    x.parse().map_or(Message::Input(NewValue::Nan(x)), |x| {
                        Message::Input(NewValue::Number(call_back(x)))
                    })
                })
                .width(Length::FillPortion(10 - INPUT_WIDTH_PORTION)),
            );
        let mut c = Column::new()
            .spacing(20)
            .align_items(Align::Start)
            .width(Length::Fill)
            .push(desc);
        if let Some(Range {
            lower,
            higher,
            input_lower,
            input_slide,
            input_higher,
        }) = self.range.as_mut()
        {
            let call_back_builder = |x: SlideMessage| {
                move |new: String| {
                    new.parse().map_or(Message::Input(NewValue::Nan(new)), |n| {
                        Message::Slide((NewValue::Number(call_back(n)), x))
                    })
                }
            };
            c = c.push(
                Row::new()
                    .align_items(Align::Center)
                    .push(
                        TextInput::new(
                            input_lower,
                            "lower bound",
                            &lower.to_string(),
                            call_back_builder(SlideMessage::SetMin),
                        )
                        .width(Length::FillPortion(1)),
                    )
                    .push(
                        Slider::new(
                            input_slide,
                            RangeInclusive::new(*lower, *higher),
                            extract_property_value(value),
                            move |new: f64| {
                                Message::Slide((
                                    NewValue::Number(call_back(new)),
                                    SlideMessage::SetVal,
                                ))
                            },
                        )
                        .width(Length::Fill)
                        .step((*higher - *lower) / 1000.)
                        .width(Length::FillPortion(INPUT_WIDTH_PORTION)),
                    )
                    .push(
                        TextInput::new(
                            input_higher,
                            "higher bound",
                            &higher.to_string(),
                            call_back_builder(SlideMessage::SetMax),
                        )
                        .width(Length::FillPortion(1)),
                    ),
            );
        }
        c.into()
    }
}

#[derive(Default)]
pub struct MyChart {
    data: Vec<Vec<f64>>,
    plot: RefCell<RawAnimator>,
    map: RawMapVisualizer,
}

impl Chart<Message> for MyChart {
    fn build_chart<DB: DrawingBackend>(&self, _builder: ChartBuilder<DB>) {
        unreachable!()
    }
    #[allow(unused)]
    fn draw_chart<DB: DrawingBackend>(&self, root: DrawingArea<DB, Shift>) {
        let (upper, lower) = root.split_vertically(50.percent());
        if let Some(d) = self.data.last() {
            RefCell::borrow_mut(&self.plot)
                .new_frame_on(d.iter().enumerate().map(|(x, y)| (x as f64, *y)), &upper)
                .unwrap();
            self.map.draw_on(&self.data, &lower).unwrap();
        } else {
            warn!("trying drawing empty data");
        }
    }
}

impl MyChart {
    pub fn push(&mut self, new_data: Vec<f64>) {
        self.map.update_range(&new_data);
        self.data.push(new_data);
    }
    pub fn data_mut(&mut self) -> &mut Vec<Vec<f64>> {
        &mut self.data
    }
    pub fn get_data(&self) -> &Vec<Vec<f64>> {
        &self.data
    }
    pub fn view(&mut self) -> Element<Message> {
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
