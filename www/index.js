import { Worker, WorkerProperty, CursorPos, Point } from "lle-simulator";

const canvas = document.getElementById("plot");

var worker;

const newWorker = () => {
    worker = Worker.new("plot");
};


const getProperty = () => {
    return worker.get_property();
};
const setProperty = (name, value) => {
    worker.set_property(name, value);
};


function updatePlot() {
    worker.tick();
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

/** Setup canvas to properly handle high DPI and redraw current plot. */
function setupCanvas() {
    const dpr = window.devicePixelRatio || 1.0;
    const aspectRatio = canvas.width / canvas.height;
    const size = canvas.parentNode.offsetWidth * 0.8;
    canvas.style.width = size + "px";
    canvas.style.height = size / aspectRatio + "px";
    canvas.width = size;
    canvas.height = size / aspectRatio;
}


let animationId = null;
const isPaused = () => {
    return animationId === null;
};



const renderLoop = () => {
    updatePlot();
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
    updatePlot();
}

playPauseButton.onclick = (event => {
    if (isPaused()) {
        play();
    } else {
        pause();
    }
});


window.onload = function () {
    setupCanvas();
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
    updatePlot();
    pause();
}
window.onresize = function () {
    updatePlot();
}