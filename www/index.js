import { Worker } from "wasm-lle";
import { memory } from "wasm-lle/wasm_lle_bg"
import {
    Chart,
    ArcElement,
    LineElement,
    BarElement,
    PointElement,
    BarController,
    BubbleController,
    DoughnutController,
    LineController,
    PieController,
    PolarAreaController,
    RadarController,
    ScatterController,
    CategoryScale,
    LinearScale,
    LogarithmicScale,
    RadialLinearScale,
    TimeScale,
    TimeSeriesScale,
    Decimation,
    Filler,
    Legend,
    Title,
    Tooltip,
    SubTitle
} from 'chart.js';

Chart.register(
    ArcElement,
    LineElement,
    BarElement,
    PointElement,
    BarController,
    BubbleController,
    DoughnutController,
    LineController,
    PieController,
    PolarAreaController,
    RadarController,
    ScatterController,
    CategoryScale,
    LinearScale,
    LogarithmicScale,
    RadialLinearScale,
    TimeScale,
    TimeSeriesScale,
    Decimation,
    Filler,
    Legend,
    Title,
    Tooltip,
    SubTitle
);const input = document.getElementById("input");

const worker = Worker.new();
const getProperty = () => {
    let data = worker.get_property();
    console.log("js get value", data, typeof (data));
    return data;
};
const setProperty = (value) => {
    console.log("js set value", value, typeof (value));
    worker.set_property(value);
};

document.getElementById("but1").onclick = uploadValue;


const labels = new Float64Array(memory.buffer, worker.get_property(), worker.get_len());

const canvas = document.getElementById("plot");
const ctx = canvas.getContext('2d');

const data = {
    labels: labels,
    datasets: [{
        label: 'My First dataset',
        backgroundColor: 'rgb(255, 99, 132)',
        borderColor: 'rgb(255, 99, 132)',
        data:[1],
    }]
};
data.datasets[0].data = new Float64Array(memory.buffer, worker.get_property(), worker.get_len());
const config = {
    type: 'line',
    data: data,
    options: {}
};

const plot = () => {
    let chartStatus = Chart.getChart("plot");
    if (chartStatus != undefined) {
        chartStatus.destroy();
    }
    console.log(data);
    const myChart = new Chart(ctx, config);
}

plot();

function uploadValue() {
    plot();
}
