# bee-node

# Installing from source

## Dependencies

### Debian

```sh
apt-get update
apt-get upgrade
apt-get install git build-essential cmake pkg-config librocksdb-dev llvm clang libclang-dev libssl-dev
```

### MacOS

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install cmake
```

### Windows

Open Powershell and execute the following commands:
```sh
Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))
choco install git --params '/NoAutoCrlf' cmake --installargs 'ADD_CMAKE_TO_PATH=System' llvm
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
git clone https://github.com/iotaledger/bee.git --branch mainnet-develop
cd bee/bee-node
```

With dashboard

```sh
cargo build --release --features dashboard
```

Without dashboard
```sh
cargo build --release
```

## Running

```sh
cp config.template.json config.json
../target/release/bee
```

# Using Docker

We also provide a `Dockerfile` that allows you to quickly deploy a Bee node. Please refer to the [Docker](../documentation/docs/getting_started/docker.md) section of the Bee documentation for more information.
