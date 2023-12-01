const path = require("path");
const CopyWebpackPlugin = require("copy-webpack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  entry: {
    index: "./js/index.js",
  },
  output: {
    path: dist,
    filename: "[name].js",
    clean: true,
  },
  devServer: {
    static: {
      directory: dist,
    },
    compress: true,
  },
  plugins: [
    new CopyWebpackPlugin({
      patterns: [path.resolve(__dirname, "static")],
    }),
  ],
  module: {
    rules: [
      {
        test: /\.css$/i,
        use: ["style-loader", "css-loader"],
      },
    ],
  },
  experiments: {
    asyncWebAssembly: true,
  },
};
