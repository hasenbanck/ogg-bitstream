# ogg-ng

[![Latest version](https://img.shields.io/crates/v/ogg-ng.svg)](https://crates.io/crates/ogg-ng)
[![Documentation](https://docs.rs/ogg-ng/badge.svg)](https://docs.rs/ogg-ng)
![ZLIB](https://img.shields.io/badge/license-zlib-blue.svg)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)

Reads and writes OGG container streams.

## Overview

Implements the following specifications:

* [rfc3533](https://tools.ietf.org/html/rfc3533.html)
* [rfc7845](https://tools.ietf.org/html/rfc7845.html)

Following media mappings are currently supported:

* Opus
* Vorbis

Further media mappings can be added once Rust native codec implementations are available.

## Features

All the features are enabled by default.

* "decoder": Enables the decoder / bitstream reader.
* "encoder": Enables the encoder / bitstream writer.
* "opus": Enables the Vorbis media mapping.
* "vorbis": Enables the Vorbis media mapping.

## License

Licensed under MIT or Apache-2.0 or ZLIB.
