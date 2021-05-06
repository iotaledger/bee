# bee-node

# Installing from source

## Dependencies

### Debian

```sh
apt-get update
apt-get upgrade
apt-get install git npm build-essential cmake pkg-config librocksdb-dev llvm clang libclang-dev libssl-dev
```

### MacOS

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install cmake npm
```

### Windows

Open Powershell and execute the following commands:
```sh
Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))
choco install git --params '/NoAutoCrlf' nodejs-lts cmake --installargs 'ADD_CMAKE_TO_PATH=System' llvm
```
Restart Powershell

### Rust

Minimum required version 1.48.

#### Installation (Debian, MacOS)

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Installation (Windows)

Install Rust from [here](https://www.rust-lang.org/learn/get-started).

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
cd ../../../../
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

## Script

There is a `build_and_run.sh` script that will build the bee node with dashboard or without, it also builds the `docker image` and run the container using `docker-compose`

- usage
  - `build bee` build without the dashboard
  - `build bee-dashboard` build with dashboard
  - `build docker`  create a docker image using as image tag the version in `Cargo.toml` 
  - `run bee-image`  run a bee node container
  - `-h` or ` help` print help