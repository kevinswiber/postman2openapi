const path = require('path');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

const dist = path.resolve(__dirname, 'dist');

module.exports = {
  entry: {
    index: './js/index.js',
  },
  output: {
    path: dist,
    filename: '[name].js',
  },
  devServer: {
    contentBase: dist,
  },
  plugins: [
    new CleanWebpackPlugin(),
    new CopyWebpackPlugin([path.resolve(__dirname, 'static')]),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, '..'),
      outDir: path.resolve(__dirname, 'wasm'),
    }),
  ],
  module: {
    rules: [
      {
        test: /\.css$/i,
        use: ['style-loader', 'css-loader'],
      },
    ],
  },
};
