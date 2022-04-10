use std::{
    mem::size_of,
    ops::RangeInclusive,
    thread::{spawn, JoinHandle},
};

use iced::{
    slider, text_input, Align, Column, Element, HorizontalAlignment, Length, Row, Slider, Text,
    TextInput,
};
use jkplot::{RawAnimator, RawMapVisualizer};
use lle_simulator::WorkerUpdate;
use minifb::{Scale, Window, WindowOptions};
use plotters::prelude::*;

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
                        Container::new(
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
                            .step((*higher - *lower) / 10000.),
                        )
                        .width(Length::FillPortion(INPUT_WIDTH_PORTION))
                        .padding(5),
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

fn u32_to_u8(arr: &mut [u32]) -> &mut [u8] {
    let len = size_of::<u32>() / size_of::<u8>() * arr.len();
    let ptr = arr.as_mut_ptr() as *mut u8;
    unsafe { std::slice::from_raw_parts_mut(ptr, len) }
}

pub struct DrawData {
    data: Vec<Vec<f64>>,
    plot: RawAnimator,
    map: SpawnMapVisual,
    window: Option<Window>,
    size: (usize, usize),
    buffer: Vec<u32>,
}

struct StandBy {
    bitmap_buffer: Vec<u32>,
    data: Vec<Vec<f64>>,
    map: RawMapVisualizer,
    size: (usize, usize),
}

#[allow(unused)]
impl StandBy {
    fn new(size: (usize, usize)) -> Self {
        Self {
            bitmap_buffer: vec![0; size.0 * size.1],
            data: Vec::new(),
            map: RawMapVisualizer::default(),
            size,
        }
    }
    fn set_size(&mut self, size: (usize, usize)) {
        self.bitmap_buffer.resize(size.0 * size.1, 0);
        self.size = size;
    }
    fn draw(&mut self) -> Result<()> {
        self.map.draw_on(
            &self.data,
            &BitMapBackend::<plotters_bitmap::bitmap_pixel::BGRXPixel>::with_buffer_and_format(
                u32_to_u8(&mut self.bitmap_buffer),
                (self.size.0 as u32, self.size.1 as u32),
            )?
            .into_drawing_area(),
        )?;
        Ok(())
    }
    fn spawn(mut self) -> JoinHandle<Self> {
        spawn(move || {
            self.draw().expect("failed drawing color map");
            self
        })
    }
}

enum SpawnMapVisual {
    StandBy(StandBy),
    Handler(JoinHandle<StandBy>),
    ///this should never appear other than its own method for temporary take the data ownership
    Temp,
}

#[allow(unused)]
impl SpawnMapVisual {
    fn new(size: (usize, usize)) -> Self {
        SpawnMapVisual::StandBy(StandBy::new(size))
    }
    fn try_set_size(&mut self, size: (usize, usize)) -> bool {
        if let SpawnMapVisual::StandBy(s) = self {
            s.set_size(size);
            true
        } else {
            false
        }
    }
    fn try_update(&mut self, new_data: &mut Vec<Vec<f64>>, buffer_dis: &mut [u32]) -> Result<bool> {
        match std::mem::replace(self, SpawnMapVisual::Temp) {
            SpawnMapVisual::StandBy(mut s) => {
                new_data.iter().for_each(|x| {
                    s.map.update_range(x);
                });
                s.data.append(new_data);
                buffer_dis.clone_from_slice(&s.bitmap_buffer);
                *self = SpawnMapVisual::Handler(s.spawn());
                Ok(true)
            }
            SpawnMapVisual::Handler(h) => {
                if h.is_finished() {
                    *self = SpawnMapVisual::StandBy(
                        h.join().map_err(|_| anyhow!("color map thread panicked"))?,
                    );
                    Ok(true)
                } else {
                    *self = SpawnMapVisual::Handler(h);
                    Ok(false)
                }
            }
            SpawnMapVisual::Temp => unreachable!(),
        }
    }
}

impl DrawData {
    fn split_area(size: usize) -> (usize, usize) {
        (size / 2, size - size / 2)
    }
    pub fn new(size: (usize, usize)) -> Self {
        DrawData {
            data: Vec::default(),
            plot: RawAnimator::default(),
            map: SpawnMapVisual::new((size.0, Self::split_area(size.1).1)),
            window: None,
            size,
            buffer: Vec::default(),
        }
    }
    #[allow(unused)]
    pub fn update(&mut self) -> Result<()> {
        //get or create window
        let size = self.size;
        let window = match self.window {
            Some(ref mut w) => {
                if !w.is_open() {
                    *w = Window::new(
                        "Status display",
                        size.0,
                        size.1,
                        WindowOptions {
                            scale: Scale::X1,
                            ..WindowOptions::default()
                        },
                    )?;
                }
                w
            }
            _ => {
                self.buffer.resize(size.0 * size.1, 0);
                self.window.insert(
                    (Window::new(
                        "Status dispay",
                        size.0,
                        size.1,
                        WindowOptions {
                            scale: Scale::X1,
                            ..WindowOptions::default()
                        },
                    )?),
                )
            }
        };
        //draw chart
        {
            let (upper_size, lower_size) = Self::split_area(size.1);
            let (upper_buffer, lower_buffer) =
                self.buffer[..].split_at_mut(size.0 * upper_size as usize);
            let upper =
                BitMapBackend::<plotters_bitmap::bitmap_pixel::BGRXPixel>::with_buffer_and_format(
                    u32_to_u8(upper_buffer),
                    (size.0 as u32, upper_size as u32),
                )?
                .into_drawing_area();
            if let Some(d) = self.data.last() {
                self.plot
                    .new_frame_on(d.iter().enumerate().map(|(x, y)| (x as f64, *y)), &upper)
                    .unwrap();
                self.map.try_update(&mut self.data, lower_buffer)?;
            } else {
                warn!("trying drawing empty data");
            }
        }
        window.update_with_buffer(&self.buffer, size.0, size.1)?;
        Ok(())
    }
    pub fn push(&mut self, new_data: Vec<f64>) {
        self.data.push(new_data);
    }
}
