<h1 align="center">🎨 Welcome to Color Buddy 🎨</h1>
<p align="center">
  <img alt="Version" src="https://img.shields.io/badge/version-0.1.6-blue.svg?cacheSeconds=2592000" />
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
- [FAQs](#faqs)
- [Help](#help)
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
  -o, --output <OUTPUT>

  -t, --output-type <OUTPUT_TYPE>
          [default: original-image] [possible values: json, original-image, standalone-palette]
  -p, --palette-height <PALETTE_HEIGHT>
          [default: 256]
  -w, --palette-width <PALETTE_WIDTH>

  -h, --help
          Print help information
  -V, --version
          Print version information
```

## Examples

The default options will result in:
  - a copy of the original image being output with a palette of
  - 8 colors along the bottom
  - with a height of 256px
  - calculated using k-means clustering

**Generate JSON containing the 8 most prevalent colors in the image:**
```sh
colorbuddy --output-type json original-image.jpg
```

**Output the original images with a palette of the 5 most prevalent colors along the bottom:**
```sh
colorbuddy --number-of-colors 5 --output-type original-image.jpg another-image.jpg
```

**Specify the height of the palette as a percentage of the original image's height:**
```sh
colorbuddy --palette-height 20% original-image.jpg
```

**Specify a width, height, and the standalone-palette output height to create a standalone palette image:**
```sh
colorbuddy --palette-height 50px --palette-width 500 original-image.jpg
```

## FAQs

<details>
  <summary>Q. What is Median Cut Quantisation?</summary>

[Median Cut quantization](https://en.wikipedia.org/wiki/Median_cut) is a method used in image processing to reduce the number of colors used in an image. The goal is to represent the original image using a smaller color palette, while preserving as much of the visual information as possible.

Think of it like this: imagine you have a box of crayons, and you want to reduce the number of crayons you have while still being able to color a picture. The Median Cut quantization method would help you choose a smaller set of crayons that represent the range of colors used in your picture, so that you can still color a picture that looks similar to the original.

In Median Cut quantization, the first step is to divide the color space of the image into smaller sections. This is done by finding the median color value in each section and dividing the section in two based on this median value. This process is repeated until you have the desired number of colors in your palette.

</details>

<details>
<summary>Q. What is "k-means clustering"?</summary>

[K-means clustering](https://en.wikipedia.org/wiki/K-means_clustering) is a machine learning technique used for grouping data into "clusters" based on similarities between the data points.

Think of it like this: imagine you have a bunch of different colored balls, and you want to group them into a few different baskets based on their color. K Means Clustering is a way for the computer to automatically separate the balls into baskets such that each basket contains balls of similar color.

The "K" in K Means refers to the number of baskets you want to create. So, you can choose to have 2 baskets, 3 baskets, or even 10 baskets, depending on how many groups you want to create.

</details>

<details>
  <summary>Q. What is the difference between the two quantisation methods?</summary>

Or: "When should I use one method over the other?"

K-means clustering results in a palette of the **most common** colours in the image, whereas median cut quantisation results in a palette of **representative** colours.

Experiment with what works best for your application!

</details>

## Help

This is a personal, unpaid, open source project.
If you encounter a bug or other issue, require help or the addition of some new feature, please feel free to raise an Issue on GitHub.
I will endeavour to respond in a relatively timely manner but provide no guarantees.

## Roadmap

- [ ] Allow users to generate palette information used by their graphics tools/applications
- [ ] Refactor and pay down technical debt
- [x] ~~Allow users to generate a separate standalone palette image~~
- [x] ~~Allow users to specify multiple images upon which to apply the same options~~
- [x] ~~Allow users to specify an output file/directory~~
- [x] ~~Add tests~~

## Author

👨 **Adam Henley (he/him)**

* Website: [https://adamhenley.com](https://adamhenley.com)
* Twitter: [@adamofgreyskull](https://twitter.com/adamofgreyskull)
* GitHub: [@adamazing](https://github.com/adamazing)
* LinkedIn: [@adamhenley](https://linkedin.com/in/adamhenley)

## Show your support

Give a ⭐️ if this project helped you!

