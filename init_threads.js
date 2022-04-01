function onInit(wasm) {
    wasm.initThreadPool(navigator.hardwareConcurrency);;
}
export default onInit;