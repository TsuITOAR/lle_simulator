use lle_simulator::*;
use log::info;
use std::time::{Duration, Instant};
use wasm_bindgen::prelude::*;
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
        let ControlProps {
            desc,
            slide_bar,
            id,
            init: current,
            call_back,
        } = ctx.props();

        let link = ctx.link().clone();
        let onchange = call_back.reform(move |e: web_sys::Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let v = input.value_as_number();
            link.send_message(SlideBarMessage::SetVal(v));
            v
        });

        let link = ctx.link().clone();
        let oninput = call_back.reform(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let v = input.value_as_number();
            link.send_message(SlideBarMessage::SetVal(v));
            v
        });

        let set_max = ctx.link().callback(|e: web_sys::Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            SlideBarMessage::SetMax(input.value_as_number())
        });
        let set_min = ctx.link().callback(|e: web_sys::Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            SlideBarMessage::SetMin(input.value_as_number())
        });
        let value = ctx.props().init;
        let min = value - 5.;
        let max = value + 5.;
        {
            if let Some(c) = self.range.as_ref() {
                html! {
                    <div class="control">
                        <div class="desc">
                            <label for={id.clone()} class="item_label"> {desc} </label>
                            <input class="current" id={id.clone()} type="number" value={value.to_string()} {onchange} ref={c.current.clone()}/>
                        </div>
                        <div class="slide">
                            <input class="bound" id={format!("{}_min",id.clone())} type="number" onchange={set_min} value={min.to_string()}/>
                            <input class="bar"   id={format!("{}_slide",id.clone())} type="range" step="any" min={min.to_string()} max={max.to_string()} value={value.to_string()} {oninput} ref={c.bar.clone()}/>
                            <input class="bound" id={format!("{}_max",id.clone())} type="number" onchange={set_max} value={max.to_string()}/>
                        </div>
                    </div>
                }
            } else {
                html! {
                    <div class="control">
                            <div class="desc">
                                <label for={id.clone()} class="item_label"> {desc} </label>
                                <input class="current" id={id.clone()} type="number" value={value.to_string()} {onchange}/>
                            </div>
                    </div>
                }
            }
        }
    }
}

const PLOT_NAME: &str = "plot";
const MAP_NAME: &str = "map";

struct App {
    worker: Option<Worker>,
    last_frame: Option<Instant>,
}

enum WorkerPara {
    Alpha(f64),
    Pump(f64),
    Linear(f64),
    RecordStep(u64),
    SimuStep(f64),
}

impl Component for App {
    type Message = WorkerPara;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            worker: None,
            last_frame: None,
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }
    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.worker = Worker::new(PLOT_NAME, MAP_NAME).into();
            //self.last_frame = Instant::now().into();
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
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
                            <Control desc={*x} id={*x} slide_bar={true} init={1.} call_back={link.callback( WorkerPara::Alpha)}/>
                            }
                        )
                        .collect::<Html>()
                    }
                    {INPUT_STATIC
                        .iter()
                        .map(|x| html!{
                            <Control desc={*x} id={*x} slide_bar={false} init={1.} call_back={link.callback( WorkerPara::Alpha)}/>
                            }
                        )
                        .collect::<Html>()
                    }
                </div>
            </div>
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    yew::start_app_with_props::<App>(());
}
