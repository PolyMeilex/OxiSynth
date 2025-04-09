const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const { experiments } = require("webpack");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "production",
  experiments: {
    syncWebAssembly: true
  },
  entry: {
    index: "./index.js"
  },
  output: {
    path: dist,
    filename: "[name].js"
  },
  devServer: {
    static: {
      directory: dist,
    },
    open: true,
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: 'index.html'
    }),
    new WasmPackPlugin({
      crateDirectory: __dirname
    }),
  ]
};
