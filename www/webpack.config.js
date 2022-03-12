const CopyWebpackPlugin = require("copy-webpack-plugin");
const WasmPackWatcherPlugin = require('wasm-pack-watcher-plugin')
const path = require('path');

module.exports = {
    entry: "./bootstrap.js",
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "bootstrap.js",
    },
    mode: "development",
    plugins: [
        new WasmPackWatcherPlugin({
            sourceRoot: path.resolve('..', 'src'),
            crateRoot: path.resolve('..'),
        })
    ]
};
