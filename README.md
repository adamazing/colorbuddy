<h1 align="center">🎨 Welcome to Color Buddy 🎨</h1>
<p>
  <img alt="Version" src="https://img.shields.io/badge/version-0.1.4-blue.svg?cacheSeconds=2592000" />
  <a href="#" target="_blank">
    <img alt="License: MIT" src="https://img.shields.io/badge/License-MIT-yellow.svg" />
  </a>
  <a href="https://twitter.com/adamofgreyskull" target="_blank">
    <img alt="Twitter: adamofgreyskull" src="https://img.shields.io/twitter/follow/adamofgreyskull.svg?style=social" />
  </a>
</p>

> Color Buddy is a command line tool that can generate a color palette from an image.

## Contents

<!--toc:start-->
- [Contents](#contents)
- [Install](#install)
- [Usage](#usage)
- [Examples](#examples)
- [Roadmap](#roadmap)
- [Author](#author)
- [Show your support](#show-your-support)
<!--toc:end-->

## Install

```sh
cargo install colorbuddy
```

## Usage

```sh
colorbuddy --help
```

Produces the following output:
```
Generates a color palette based on an image.

Usage: colorbuddy [OPTIONS] <IMAGE>

Arguments:
  <IMAGE>

Options:
  -m, --quantisation-method <QUANTISATION_METHOD>
          [default: k-means] [possible values: median-cut, k-means]
  -n, --number-of-colors <NUMBER_OF_COLORS>
          [default: 8]
  -t, --output-type <OUTPUT_TYPE>
          [default: original-image] [possible values: json, original-image]
  -p, --palette-height <PALETTE_HEIGHT>
          [default: 256]
  -h, --help
          Print help
  -V, --version
          Print version
```

## Examples

The default options will result in:
  - a copy of the original image being output with a palette of
  - 8 colors along the bottom
  - with a height of 256px
  - calculated using k-means clustering

Generate a JSON file containing the 8 most prevalent colors in the image:
```sh
colorbuddy --output-type json original-image.jpg
```

Output the original images with a palette of the 5 most prevalent colors along the bottom:
```sh
colorbuddy --number-of-colors 5 --output-type original-image.jpg another-image.jpg
```

## Roadmap

- [x] ~~Allow users to specify multiple images upon which to apply the same options~~
- [ ] Allow users to specify an output file/directory
- [ ] Allow users to generate a separate standalone palette image
- [ ] Allow users to generate palette information used by their graphics tools/applications
- [ ] Add tests

## Author

👨 **Adam Henley (he/him)**

* Website: https://adamhenley.com
* Twitter: [@adamofgreyskull](https://twitter.com/adamofgreyskull)
* Github: [@adamazing](https://github.com/adamazing)
* LinkedIn: [@adamhenley](https://linkedin.com/in/adamhenley)

## Show your support

Give a ⭐️ if this project helped you!

