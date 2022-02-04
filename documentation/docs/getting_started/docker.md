---
keywords:
- IOTA Node
- Bee Node
- Docker
- Install
- Run
- macOS
- Windows
- Linux
description: Install and run a Bee node using Docker.
image: /img/logo/bee_logo.png
---

# Using Docker

Bee Docker images (amd64/x86_64 architecture) are available at [iotaledger/bee](https://hub.docker.com/r/iotaledger/bee) on Docker Hub.

## Requirements

1. A recent release of Docker enterprise or community edition. You can finde installation instructions in the [official Docker documentation](https://docs.docker.com/engine/install/).
2. [GIT](https://git-scm.com/)
4. At least 1GB available RAM

## Clone the Repository

Once you have completed all the installation [requirements](#requirements), you can clone the repository by running:

```sh
git clone https://github.com/iotaledger/bee
cd bee/
```
:::info
The next portion of the guide assumes you are executing commands from the root directory of the repository.
:::

## Prepare

Choose the `config.toml` for the network that you want to join: and copy it to `/.config.toml`:

```sh
cp ./config.chrysalis-<insert network here>.toml config.toml. 
```

If you want to use alternative ports, edit the `config.toml` file.

## Run

You can pull the latest image from `iotaledger/bee` public Docker Hub registry by running:

```sh
docker pull iotaledger/bee:latest && docker tag iotaledger/bee:latest bee:latest
```

We recommend that you run on host network to improve performance.  Otherwise, you are going to have to publish ports using iptables NAT which is slower.

```sh
docker run \
  -v $(pwd)/config.toml:/config.toml:ro \
  -v $(pwd)/storage:/storage \
  -v $(pwd)/snapshots:/snapshots \
  --name bee\
  --net=host \
  --ulimit nofile=8192:8192 \
  -d \
  bee:latest
```

* `$(pwd)` Stands for the present working directory. All mentioned directories are mapped to the container, so the Bee in the container persists the data directly to those directories.
* `-v $(pwd)/config.toml:/app/config.toml:ro` Maps the local `config.toml` file into the container in `readonly` mode.
* `-v $(pwd)/storage:/storage` Maps the local `storage` directory into the container.
* `-v $(pwd)/snapshots:/snapshots` Maps the local `snapshots` directory into the container.
* `--name bee` Name of the running container instance. You can refer to the given container by this name.
* `--net=host` Instructs Docker to use the host's network, so the network is not isolated. We recommend that you run on host network for better performance.  This way, the container will also open any ports it needs on the host network, so you will not need to specify any ports.
* `--ulimit nofile=8192:8192` increases the ulimits inside the container. This is important when running with large databases.
* `-d` Instructs Docker to run the container instance in a detached mode (daemon).


You can run `docker stop -t 300 bee` to gracefully end the process.

## Create Username and Password for the Bee Dashboard

If you use the Bee dashboard, you need to create a secure password. Start your Bee container and execute the following command when the container is running:

```sh
docker exec -it bee password

```

Expected output:

```plaintext
Password: [enter password]
Re-enter password: [enter password]
Password salt: [password salt]
Password hash: [password hash]
```

You can edit `config.toml` and customize the _dashboard_ section to your needs.

```toml
[dashboard]
bind_address    = "/ip4/0.0.0.0/tcp/8081"
[dashboard.auth]
session_timeout = 86400
user            = "admin"
password_salt   = "[password salt]"
password_hash   = "[password hash]"
```

## Build Your Own Bee Image

You can build your own Docker image by running the following command:

```sh
docker build -f bee-node/docker/Dockerfile -t bee:latest .
```

Or pull it from Docker Hub (only available for amd64/x86_64):

```sh
docker pull iotaledger/bee:latest && docker tag iotaledger/bee:latest bee:latest
```

## Managing a Node

:::info
Bee uses an in-memory cache. In order to save all data to the underlying persistent storage, it is necessary to provide a grace period of at least 200 seconds while shutting it down.
:::

### Starting an Existing Bee Container

You can start an existing Bee container by running:

```sh
docker start bee
```

### Restarting Bee

You can restart an existing Bee container by running:

```sh
docker restart -t 300 bee
```

* `-t 300` Instructs Docker to wait for a grace period before shutting down.

### Stopping Bee

You can stop an existing Bee container by running:

```sh
docker stop -t 300 bee
```

* `-t 300` Instructs Docker to wait for a grace period before shutting down.

### Displaying Log Output

You can display an existing Bee containers logs by running:

```sh
docker logs -f bee
```

* `-f`
Instructs Docker to continue displaying the log to `stdout` until CTRL+C is pressed.

## Removing a Container

You can remove an existing Bee container by running:

```sh
docker container rm bee
```

## Setup using `docker-compose`

You can keep track of different configurations of the node using the [`docker-compose`](https://docs.docker.com/compose/). An example `docker-compose.yml` is in `./bee-node/docker/`, if you just quickly want to try out the node software on its own.

If you want to run the latest release from Docker Hub you can call:

```sh
docker-compose -f bee-node/docker/docker-compose.yml up --no-build
```

Or, if you want to build the latest version of Bee from source, you can use:

```sh
docker-compose -f bee-node/docker/docker-compose.yml up --build
```
