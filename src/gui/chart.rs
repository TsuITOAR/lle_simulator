use super::*;

use std::{
    mem::{self, size_of},
    sync::Arc,
    thread::{spawn, JoinHandle},
};

use jkplot::{RawAnimator, RawMapVisualizer};
use lle::num_complex::Complex64;
use minifb::{Scale, Window, WindowOptions};
use plotters::prelude::*;
use rustfft::{Fft, FftPlanner};

fn u32_to_u8(arr: &mut [u32]) -> &mut [u8] {
    let len = size_of::<u32>() / size_of::<u8>() * arr.len();
    let ptr = arr.as_mut_ptr() as *mut u8;
    unsafe { std::slice::from_raw_parts_mut(ptr, len) }
}

pub struct DrawData {
    data: Vec<Vec<Complex64>>,
    plot_real: RawAnimator,
    plot_freq: RawAnimator,
    fft: Arc<dyn Fft<f64>>,
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
    fn try_update(
        &mut self,
        new_data: &mut Vec<Vec<Complex64>>,
        buffer_dis: &mut [u32],
    ) -> Result<bool> {
        match std::mem::replace(self, SpawnMapVisual::Temp) {
            SpawnMapVisual::StandBy(mut s) => {
                s.data.reserve(new_data.len());
                new_data.iter_mut().for_each(|x| {
                    let temp = mem::take(x).into_iter().map(|x| x.re).collect::<Vec<_>>();
                    s.map.update_range(&temp);
                    s.data.push(temp);
                });
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
    pub fn new(data_len: usize, window_size: (usize, usize)) -> Self {
        DrawData {
            data: Vec::default(),
            plot_real: RawAnimator::default(),
            plot_freq: {
                let mut a = RawAnimator::default();
                a.set_y_desc("dB");
                a.set_x_label_formatter(move |x| format!("{}", (x - (data_len / 2) as f64)));
                a
            },
            fft: FftPlanner::new().plan_fft_forward(data_len),
            map: SpawnMapVisual::new((window_size.0, Self::split_area(window_size.1).1)),
            window: None,
            size: window_size,
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
            let lower =
                BitMapBackend::<plotters_bitmap::bitmap_pixel::BGRXPixel>::with_buffer_and_format(
                    u32_to_u8(lower_buffer),
                    (size.0 as u32, lower_size as u32),
                )?
                .into_drawing_area();
            if let Some(d) = self.data.last() {
                self.plot_real
                    .new_frame_on(d.iter().enumerate().map(|(x, y)| (x as f64, y.re)), &upper)
                    .unwrap();
                let mut freq = d.to_owned();
                self.fft.process(&mut freq);
                let (first, second) = freq.split_at(freq.len() / 2);

                self.plot_freq
                    .new_frame_on(
                        second
                            .iter()
                            .chain(first.iter())
                            .enumerate()
                            .map(|(x, y)| (x as f64, 10. * (y.norm().log10()))),
                        &lower,
                    )
                    .unwrap();
                //self.map.try_update(&mut self.data, lower_buffer)?;
            } else {
                warn!("trying drawing empty data");
            }
        }
        window.update_with_buffer(&self.buffer, size.0, size.1)?;
        Ok(())
    }
    pub fn push(&mut self, new_data: Vec<Complex64>) {
        self.data.push(new_data);
    }
}
