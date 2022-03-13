import { Worker, WorkerProperty } from "wasm-lle";
import { memory } from "wasm-lle/wasm_lle_bg"
import Plotly from 'plotly.js-dist-min'

const worker = Worker.new();
const getProperty = () => {
    return worker.get_property();
};
const setProperty = (name, value) => {
    worker.set_property(name, value);
};

setProperty("record_step", 1000);

const getData = () => {
    let data = new Float64Array(memory.buffer, worker.get_state(), worker.get_len());
    return data;
}




const updatePlot = () => {
    Plotly.react(document.getElementById("plot"), [{
        y: getData()
    }])
}

const newPlot = () => {
    Plotly.newPlot(document.getElementById("plot"), [{
        y: getData()
    }], {
        margin: { t: 0 }
    }, { displaylogo: false });
}

const updateDraw = () => {
    worker.tick();
    updatePlot()
}


let animationId = null;
const isPaused = () => {
    return animationId === null;
};
const renderLoop = () => {
    updateDraw()
    animationId = requestAnimationFrame(renderLoop);
};
const playPauseButton = document.getElementById("play-pause");
const stepButton = document.getElementById("step");

const play = () => {
    playPauseButton.textContent = "Pause";
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
                    console.log("resetting lower limit from", lower.value, "to", current.value);
                    lower.value = current.value;
                    bar.min = current.value;
                }
                if (parseFloat(current.value) > parseFloat(higher.value)) {
                    console.log("resetting higher limit from", higher.value, "to", current.value);
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