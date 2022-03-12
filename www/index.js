import { Worker, WorkerProperty } from "wasm-lle";
import { memory } from "wasm-lle/wasm_lle_bg"
import Plotly from 'plotly.js-dist-min'

const input = document.getElementById("input");

const worker = Worker.new();
const getProperty = () => {
    let data = worker.get_property();
    return data;
};
const setProperty = (name, value) => {
    worker.set_property(name, value);
};

setProperty("record_step", 1000);

const getData = () => {
    let data = new Float64Array(memory.buffer, worker.get_state(), worker.get_len());
    return data;
}

Plotly.newPlot(document.getElementById("plot"), [{
    y: getData()
}], {
    margin: { t: 0 }
});


const plot = () => {
    Plotly.react(document.getElementById("plot"), [{
        y: getData()
    }])
}


const updateDraw = () => {
    worker.tick();
    plot()
}
document.getElementById("but1").onclick = updateDraw;