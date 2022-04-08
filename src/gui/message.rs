use std::{mem::size_of, ops::RangeInclusive};

use iced::{
    slider, text_input, Align, Column, Element, HorizontalAlignment, Length, Row, Slider, Text,
    TextInput,
};
use jkplot::{RawAnimator, RawMapVisualizer};
use lle_simulator::WorkerUpdate;
use minifb::{Scale, Window, WindowOptions};
use plotters::{prelude::*, style::AsRelative};

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
                        .step((*higher - *lower) / 10000.)
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
    plot: RawAnimator,
    map: RawMapVisualizer,
    window: Option<(Window, (usize, usize))>,
    buffer: Vec<u32>,
}

impl MyChart {
    const WIDTH: usize = 640;
    const HEIGHT: usize = 640;
    #[allow(unused)]
    pub fn update(&mut self) -> Result<()> {
        //get or create window
        let (w, size) = match self.window {
            Some((ref mut w, (width, height))) => {
                if !w.is_open() {
                    *w = Window::new(
                        "Status dispay",
                        width,
                        height,
                        WindowOptions {
                            scale: Scale::X2,
                            ..WindowOptions::default()
                        },
                    )?;
                }
                (w, (width, height))
            }
            _ => {
                let r = self.window.insert((
                    Window::new(
                        "Status dispay",
                        Self::WIDTH,
                        Self::HEIGHT,
                        WindowOptions {
                            scale: Scale::X2,
                            ..WindowOptions::default()
                        },
                    )?,
                    (Self::WIDTH, Self::HEIGHT),
                ));
                self.buffer.resize(r.1 .0 * r.1 .1, 0);
                (&mut r.0, r.1)
            }
        };

        //draw chart
        {
            fn u32_to_u8(arr: &mut [u32]) -> &mut [u8] {
                let len = size_of::<u32>() / size_of::<u8>() * arr.len();
                let ptr = arr.as_mut_ptr() as *mut u8;
                unsafe { std::slice::from_raw_parts_mut(ptr, len) }
            }
            let db =
                BitMapBackend::<plotters_bitmap::bitmap_pixel::BGRXPixel>::with_buffer_and_format(
                    u32_to_u8(&mut self.buffer),
                    (size.0 as u32, size.1 as u32),
                )?;
            let (upper, lower) = db.into_drawing_area().split_vertically(50.percent());
            if let Some(d) = self.data.last() {
                self.plot
                    .new_frame_on(d.iter().enumerate().map(|(x, y)| (x as f64, *y)), &upper)
                    .unwrap();
                self.map.draw_on(&self.data, &lower).unwrap();
            } else {
                warn!("trying drawing empty data");
            }
        }

        w.update_with_buffer(&self.buffer, size.0, size.1)?;
        Ok(())
    }
    pub fn push(&mut self, new_data: Vec<f64>) {
        self.map.update_range(&new_data);
        self.data.push(new_data);
    }
}
