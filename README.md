# async-bacnet

Async Rust library for [BACnet](http://www.bacnet.org/) protocol communication, built as a thin wrapper around [embedded-bacnet](https://github.com/sensinov/embedded-bacnet) using [Tokio](https://tokio.rs/) for async I/O.

## Features

- **Read/Write properties** on BACnet objects over BACnet/IP (UDP)
- **Read multiple properties** in a single request
- **Device discovery** via WHO-IS broadcast
- **bacnet-cli** — optional command-line tool for quick BACnet interactions

## Using as a library

Add to your `Cargo.toml`:

```toml
[dependencies]
async-bacnet = { git = "https://github.com/sensinov/async-bacnet.git" }
```

```rust
use async_bacnet::{Client, ReadProperty, ObjectId, ObjectType, PropertyId};

let mut client = Client::new("192.168.1.10:47808".parse().unwrap()).await?;

let request = ReadProperty::new(
    ObjectId::new(ObjectType::ObjectAnalogInput, 1),
    PropertyId::PresentValue,
);
let ack = client.read_property(request).await?;
```

## bacnet-cli

A command-line utility for reading and writing BACnet object properties.

### Install

```sh
cargo install --git https://github.com/sensinov/async-bacnet.git --features cli
```

### Usage

```
bacnet-cli <ADDRESS:PORT> <OBJECT_TYPE> <INSTANCE> [OPTIONS]
```

**Read a property:**

```sh
# Read present-value (property 85, the default) of analog-input 1
bacnet-cli 192.168.1.10:47808 object-analog-input 1

# Read a specific property by ID
bacnet-cli 192.168.1.10:47808 object-analog-input 1 -p 77
```

**Write a property:**

```sh
# Write a real value
bacnet-cli 192.168.1.10:47808 object-analog-value 3 -w 21.5 -t real

# Write a binary enumerated value
bacnet-cli 192.168.1.10:47808 object-binary-output 1 -w true -t enumerated-binary
```

**Options:**

| Flag | Description |
|------|-------------|
| `-p, --property <ID>` | Property ID to read/write (default: `85` — present-value) |
| `-w, --write-value <JSON>` | JSON value to write (requires `-t`) |
| `-t, --write-type <TYPE>` | Data type: `boolean`, `real`, `enumerated`, `enumerated-binary` |

Logging verbosity is controlled via the `RUST_LOG` environment variable (e.g. `RUST_LOG=debug`).

## Development

### Prerequisites

- Rust stable toolchain (rustc, cargo)

A [Nix flake](https://nixos.org/) is included to provide the full toolchain automatically. If you have Nix with flakes enabled:

```sh
# Enter the dev shell manually
nix develop

# Or with direnv (automatic on cd)
direnv allow
```

### Building

```sh
# Library only
cargo build

# With CLI
cargo build --features cli

# Run CLI directly
cargo run --features cli -- 192.168.1.10:47808 object-analog-input 1
```

### Running tests

```sh
cargo test
```

## License

Apache-2.0
