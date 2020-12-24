const CopyWebpackPlugin = require('copy-webpack-plugin');
const path = require('path');

module.exports = {
  mode: 'development',
  entry: './bootstrap.ts',
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
    ],
  },
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'bootstrap.js',
  },
  resolve: {
    extensions: ['.tsx', '.ts', '.js'],
  },
  plugins: [
    new CopyWebpackPlugin(['index.html']),
  ],
};
