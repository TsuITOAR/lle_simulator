use gloo_render::{request_animation_frame, AnimationFrame};
use lle_simulator::*;
use log::{error, info};
use web_sys::HtmlInputElement;
use yew::prelude::*;

const INPUT_BAR: &[&str] = &["alpha", "pump", "linear"];
const INPUT_STATIC: &[&str] = &["record_step", "simu_step"];

#[derive(Clone, Debug, Default)]
struct Connection {
    pub current: NodeRef,
    pub bar: NodeRef,
}

struct Control {
    range: Option<Connection>,
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct ControlProps {
    desc: String,
    id: String,
    #[prop_or(false)]
    slide_bar: bool,
    init: f64,
    call_back: Callback<f64>,
}

enum SlideBarMessage {
    SetMax(f64),
    SetMin(f64),
    SetVal(f64),
}

impl Component for Control {
    type Message = SlideBarMessage;

    type Properties = ControlProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            range: ctx.props().slide_bar.then(|| Default::default()),
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::info!("control updating");
        use SlideBarMessage::*;
        if let Some(c) = self.range.as_ref() {
            match msg {
                SetVal(v) => {
                    c.bar
                        .cast::<HtmlInputElement>()
                        .expect("casting to input element")
                        .set_value(&v.to_string());
                    c.current
                        .cast::<HtmlInputElement>()
                        .expect("casting to input element")
                        .set_value(&v.to_string());
                    ctx.props().call_back.emit(v);
                }
                SetMin(min) => c
                    .bar
                    .cast::<HtmlInputElement>()
                    .expect("casting to input element")
                    .set_min(&min.to_string()),

                SetMax(max) => c
                    .bar
                    .cast::<HtmlInputElement>()
                    .expect("casting to input element")
                    .set_max(&max.to_string()),
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let ControlProps { desc, id, init, .. } = ctx.props();

        let onchange = ctx.link().callback(move |e: web_sys::Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let v = input.value_as_number();
            SlideBarMessage::SetVal(v)
        });

        let oninput = ctx.link().callback(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let v = input.value_as_number();
            SlideBarMessage::SetVal(v)
        });

        let set_max = ctx.link().callback(|e: web_sys::Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            SlideBarMessage::SetMax(input.value_as_number())
        });
        let set_min = ctx.link().callback(|e: web_sys::Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            SlideBarMessage::SetMin(input.value_as_number())
        });
        let min = init - 5.;
        let max = init + 5.;
        {
            if let Some(c) = self.range.as_ref() {
                html! {
                    <div class="control">
                        <div class="desc">
                            <label for={id.clone()} class="item_label"> {desc} </label>
                            <input class="current" id={id.clone()} type="number" value={init.to_string()} {onchange} ref={c.current.clone()}/>
                        </div>
                        <div class="slide">
                            <input class="bound" id={format!("{}_min",id.clone())} type="number" onchange={set_min} value={min.to_string()}/>
                            <input class="bar"   id={format!("{}_slide",id.clone())} type="range" step="any" min={min.to_string()} max={max.to_string()} value={init.to_string()} {oninput} ref={c.bar.clone()}/>
                            <input class="bound" id={format!("{}_max",id.clone())} type="number" onchange={set_max} value={max.to_string()}/>
                        </div>
                    </div>
                }
            } else {
                html! {
                    <div class="control">
                            <div class="desc">
                                <label for={id.clone()} class="item_label"> {desc} </label>
                                <input class="current" id={id.clone()} type="number" value={init.to_string()} {onchange}/>
                            </div>
                    </div>
                }
            }
        }
    }
}

const PLOT_NAME: &str = "plot";
const MAP_NAME: &str = "map";

#[derive(Default)]
struct App {
    worker: Option<Worker>,
    last_frame: Option<f64>,
    _render_loop: Option<AnimationFrame>,
}

impl App {
    fn render_image(&mut self, ctx: &Context<Self>) {
        if let Some(ref mut worker) = self.worker {
            worker.tick().expect("rendering image");
            //request has been made to send draw message, so it's safe to drop handler
            if let Some(_) = self._render_loop.take() {
                let link = ctx.link().clone();
                self._render_loop = request_animation_frame(move |time_ms| {
                    link.send_message(AppMessage::Draw(time_ms.into()));
                })
                .into()
            }
        } else {
            error!("worker not set when try to render image")
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum AppMessage {
    Pause,
    WorkerUpdate(WorkerUpdate),
    Draw(Option<f64>),
}

impl Component for App {
    type Message = AppMessage;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Default::default()
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        info!("{:?}", msg);
        use AppMessage::*;
        match msg {
            Pause => {
                if let Some(_) = self._render_loop.take() {
                } else {
                    let link = ctx.link().clone();
                    self._render_loop = request_animation_frame(move |time_ms| {
                        link.send_message(Draw(time_ms.into()));
                    })
                    .into()
                }
            }
            Draw(l) => {
                self.last_frame = l;
                if l.is_none() {
                    self._render_loop.take();
                }
                self.render_image(ctx);
            }
            WorkerUpdate(para) => {
                if let Some(ref mut worker) = self.worker {
                    worker.set_property(para)
                } else {
                    error!("worker not set when try to update para")
                }
            }
        }
        false
    }
    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.worker = Worker::new(PLOT_NAME, MAP_NAME).into();
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        const PAUSE: &str = "Pause";
        const RUN: &str = "Run";
        let link = ctx.link();
        let pause = ctx.link().callback(|e: MouseEvent| {
            let input: web_sys::HtmlButtonElement = e.target_unchecked_into();
            if let Some(t) = input.text_content() {
                if t == PAUSE {
                    input.set_text_content(RUN.into());
                } else if t == RUN {
                    input.set_text_content(PAUSE.into());
                } else {
                    error!("Unknown text content {}", t)
                }
            } else {
                error!("can't get text content of {}", input.id())
            }
            AppMessage::Pause
        });
        let step = ctx.link().callback(|_: MouseEvent| AppMessage::Draw(None));
        html! {
            <div class="row">
                <div class="plot column">
                    <canvas id={PLOT_NAME}></canvas>
                    <canvas id={MAP_NAME}></canvas>
                </div>
                <div class="control column">
                    {INPUT_BAR
                        .iter()
                        .map(|x| html!{
                            <Control desc={*x} id={*x} slide_bar={true} init={1.} call_back={link.callback(|x| AppMessage::WorkerUpdate(WorkerUpdate::Alpha(x)))}/>
                            }
                        )
                        .collect::<Html>()
                    }
                    {INPUT_STATIC
                        .iter()
                        .map(|x| html!{
                            <Control desc={*x} id={*x} slide_bar={false} init={1.} call_back={link.callback(|x| AppMessage::WorkerUpdate(WorkerUpdate::Alpha(x)))}/>
                            }
                        )
                        .collect::<Html>()
                    }
                    <div class="run">
                        <button id="play-pause" class="flow" onclick={pause}>{RUN}</button>
                        <button id="step" class="flow" >{"Step"} </button>
                        <label id="fps" class="flow">{"FPS:"}</label>
                    </div>
                </div>
            </div>
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    jkplot::init_thread_pool(4);
    yew::start_app_with_props::<App>(());
}
