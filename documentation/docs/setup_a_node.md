# Setup a Node
You can find the source code for Bee in the [official Bee repository](https://github.com/iotaledger/bee).  Before you can install Bee from source, you will need to install some required dependencies.  

## Installing from Source

### Dependency Packages

Before starting the installation process, you should make sure your system has all the required dependencies. 

#### Debian

To run a Bee node in a Debian base system you will need to install the following packages:

- [git](https://git-scm.com/)
- [npm](https://www.npmjs.com/)
- [build-essential](https://packages.debian.org/sid/build-essential) 
- [cmake](https://cmake.org/)
- [pkg-config](https://packages.debian.org/sid/pkg-config) 
- [librocksdb-dev](https://packages.debian.org/sid/librocksdb-dev) 
- [llvm](https://apt.llvm.org/) 
- [clang](https://packages.debian.org/search?keywords=clang) 
- [libclang-dev](https://packages.debian.org/unstable/libclang-dev) 
- [libssl-dev](https://packages.debian.org/jessie/libssl-dev)

To install all of these packages, you can run the following commands:

```shell
apt-get update
apt-get upgrade
apt-get install git npm build-essential cmake pkg-config librocksdb-dev llvm clang libclang-dev libssl-dev
```

#### macOS

To run a bee node in a macOS system, you will need to install the following packages using the [brew](https://brew.sh/) package manager:

- [cmake](https://cmake.org/)
- [npm](https://www.npmjs.com/)

You can run the following command to install brew:

```shell
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

After the installer finishes, you can use brew to install the required packages by running:

```shell
brew install cmake npm
```

#### Windows

To run a bee node in a Windows system, you will need to install the following packages using the [chocolatey](https://chocolatey.org) package manager:

- [cmake](https://cmake.org/)
- [nodejs-lts](https://nodejs.org/)
- [git](https://git-scm.com/)

To install chocolatey, open Powershell and execute the following command:

```sh
Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))
```

After the installer finishes, you can use chocolatey to install the required packages by running:

```shell
choco install git --params '/NoAutoCrlf' nodejs-lts cmake --installargs 'ADD_CMAKE_TO_PATH=System' llvm
```

::info
You will need to restart Powershell for your changes to take effect.
:::

### Installing Rust

You will need to install [Rust](https://www.rust-lang.org/) in order to run a Bee node.  You should install version is [1.51](https://blog.rust-lang.org/2021/03/25/Rust-1.51.0.html), or above.


#### Debian/macOS

You can install Rust in a Debian/macOS system by running the following commands:

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Windows

You can find installation instructions for the Windows system [in the official Rust documentation](https://www.rust-lang.org/learn/get-started).

### Updating Rust

You can use [rustup](https://rustup.rs/) to update your Rust version by running the following command:

```sh
rustup update
```

## Compiling the Bee Node

### Download the Source

Once you have installed all the required dependencies, you can start compiling the Bee Node.  To do so, you can simply clone the source code by running the following command.

```shell
git clone https://github.com/iotaledger/bee.git --branch chrysalis-pt-2
```

#### Compiling
Before you start compiling Bee, you should change your current directory to `bee/bee-node`.  You can do so by running the following command from the same directory where you downloaded the source:

```shell
cd bee/bee-node
```

You can compile Bee in two manners:

### With Dashboard

If you want to build Bee with its Dashboard, you should run the following commands:

```shell
git submodule update --init
cd src/plugins/dashboard/frontend
npm install
npm run build-bee
cd ../../../../
cargo build --release --features dashboard
```

### Without dashboard

If you want to build Bee without its Dashboard, you should run the following command:

```sh
cargo build --release
```

### Running

Once you have downloaded and compiled Bee, you should copy make a copy of the example config file.  Be sure to review and update your configuration.  You can find more information on configuring bee in the [configuration section](configuration.md).

To copy the example configuration file, you should run the following command: 
```shell
cp config.example.toml config.toml
```

Once you have copied and updated the configuration, you can run your Bee node by executing the following command:

```shell
../target/release/bee
```
