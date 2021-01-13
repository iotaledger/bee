# bee-node

# Installing from source

## Dependencies

### Debian

```sh
apt-get update
apt-get upgrade
apt-get install git npm build-essential cmake pkg-config librocksdb-dev llvm clang libclang-dev
```

### MacOS

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install cmake npm
```

### Rust

Minimum required version 1.47.

#### Installation

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Update

```sh
rustup update
```

## Compilation

```sh
git clone https://github.com/iotaledger/bee.git --branch chrysalis-pt-2
cd bee/bee-node
```

With dashboard

```sh
git submodule update --init
cd src/plugins/dashboard/frontend
npm install
npm run build-bee
cd -
cargo build --release --features dashboard
```

Without dashboard
```sh
cargo build --release
```

## Running

```sh
cp config.example.toml config.toml
../target/release/bee
```
