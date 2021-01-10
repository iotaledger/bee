# bee-node

# Installing from source

## Dependencies

### Debian

```sh
apt-get install git npm build-essential cmake pkg-config librocksdb-dev llvm clang libclang-dev
```

### Rust

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

## Compilation

```sh
git clone https://github.com/iotaledger/bee.git --branch chrysalis-pt-2
cd bee/bee-node
git submodule update --init
cd src/plugins/dashboard/frontend
npm install
npm run build-bee
cd -
cargo build --release
```

## Running

```sh
cp config.example.toml config.toml
../target/release/bee
```
