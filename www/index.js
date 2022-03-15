import { Worker, WorkerProperty } from "lle-simulator";
import { memory } from "lle-simulator/lle_simulator_bg"
import Plotly from 'plotly.js-dist-min'


var inten = null;
var worker = null;
var history = [];
var his_len;
var state_len;

const newWorker = () => {
    worker = Worker.new();
    his_len = worker.history_len();
    state_len = worker.state_len();
    inten = new Float64Array(memory.buffer, worker.get_state(), state_len);

    let history_buf = new Float64Array(memory.buffer, worker.get_history(), his_len * state_len);
    for (let i = 0; i < his_len; ++i) {
        history.push(history_buf.subarray(i * state_len, i * state_len + state_len));
    }
};


const getProperty = () => {
    return worker.get_property();
};
const setProperty = (name, value) => {
    worker.set_property(name, value);
};



function newPlot() {
    Plotly.newPlot(document.getElementById("plot"), [
        {
            y: inten
        },
        {
            z: history,
            type: 'heatmap',
            transpose: true,
            xaxis: 'x2',
            yaxis: 'y2',
        }
    ], {
        margin: { t: 0 },
        grid: { rows: 1, columns: 2, pattern: 'independent' },
    }, {
        displaylogo: false
    });
}


function updatePlot() {
    Plotly.redraw(document.getElementById("plot"))
}


function updateDraw() {
    worker.tick();
    updatePlot();
}

let last = null;

function timer() {
    var dt;
    var now = new Date().getTime();
    if (last == null) { dt = 1 }
    else {
        dt = now - last;
    }
    last = now;

    return parseInt(1 / (dt / 1000));
}


let animationId = null;
const isPaused = () => {
    return animationId === null;
};



const renderLoop = () => {
    updateDraw();
    document.getElementById("fps").textContent = "FPS:" + timer();
    animationId = requestAnimationFrame(renderLoop);
};


const playPauseButton = document.getElementById("play-pause");
const stepButton = document.getElementById("step");

const play = () => {
    playPauseButton.textContent = "Pause";
    timer();
    renderLoop();
};

const pause = () => {
    playPauseButton.textContent = "Play";
    cancelAnimationFrame(animationId);
    animationId = null;
};

stepButton.onclick = () => {
    pause();
    updateDraw();
}

playPauseButton.onclick = (event => {
    if (isPaused()) {
        play();
    } else {
        pause();
    }
});
window.onload = function () {
    newWorker();
    let property = getProperty();
    ["alpha", "pump", "linear"].forEach(function (name) {
        let lower = document.getElementById(name + "_l");
        lower.value = property[name] - 5;
        let bar = document.getElementById(name);
        let higher = document.getElementById(name + "_h");
        higher.value = property[name] + 5;
        let current = document.getElementById(name + "_c");
        current.value = property[name];
        bar.min = lower.value;
        bar.max = higher.value;
        bar.value = current.value;
        lower.addEventListener("keyup", function () {
            bar.min = lower.value;
        });
        higher.addEventListener("keyup", function () {
            bar.max = higher.value;
        });
        bar.addEventListener("input", () => {
            current.value = bar.value;
            setProperty(name, bar.value);
        });
        current.addEventListener("keydown", function (event) {

            let eva = window.event || event;
            if (eva.keyCode == 13) {
                bar.value = current.value;
                setProperty(name, current.value);
                if (parseFloat(lower.value) > parseFloat(current.value)) {
                    lower.value = current.value;
                    bar.min = current.value;
                }
                if (parseFloat(current.value) > parseFloat(higher.value)) {
                    higher.value = current.value;
                    bar.max = current.value;
                }
            }
        })
    });
    ["record_step", "simu_step"].forEach(function (name) {
        let input = document.getElementById(name);
        input.value = property[name];
        input.addEventListener("keydown", function (event) {
            let eva = window.event || event;
            if (eva.keyCode == 13) {
                setProperty(name, input.value);
            }
        })
    })
    newPlot();
    pause();
}
window.onresize = function () {
    newPlot();
}