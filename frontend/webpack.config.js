const path = require('path');

module.exports = {
  entry: './src/index.tsx',
  target: "web",
  mode: "development",
  output: {
    filename: 'main.js',
    path: path.resolve(__dirname, 'dist'),
  },
  resolve: {
    extensions: [".js", ".jsx", ".json", ".ts", ".tsx"],
  },
  module: {
    rules: [
      {
        test: /\.(ts|tsx)$/,
        loader: "awesome-typescript-loader",
      },
      {
        enforce: "pre",
        test: /\.js$/,
        loader: "source-map-loader",
      },
      {
        test: /\.css$/,
        loader: "css-loader",
      },
    ],
  },
//   plugins: [
//     new HtmlWebpackPlugin({
//       template: path.resolve(__dirname, "src", "components", "index.html"),
//     }),
//     new MiniCssExtractPlugin({
//       filename: "./src/yourfile.css",
//     }),
//   ],
};