# ods-visualizer

Inspired by the idea [here](https://hackmd.io/@rootulp/celestia-hackathon-ideas#Original-Data-Square-ODS-visualizer).

This is a CLI application which subscribes to new headers from `celestia-node`, and visualizes the shares in the original data square.

Each namespace is represented by a (hopefully) unique color. The color is determined by hashing the namespace ID and using the first three bytes as the rgb values.
As such, special namespaces are also reserved accordingly.

There is also a polling based application CLI tool which can be used to poll a specific block height.

## Demo

![](https://github.com/bowenyou/ods-visualizer/blob/main/assets/demo.mp4)

## Usage

To use the subscription functionality, just run as usual

```
cargo run --release
```

To query a specific block height,

```
cargo run --release -- <BLOCK_HEIGHT>
```

Make sure that you create a config file called `config.toml` using the `example_config.toml` file as a template.

## Acknowledgements

Thanks to the folks at [trusted-point.com](https://trusted-point.com/) for access to their node while testing.
