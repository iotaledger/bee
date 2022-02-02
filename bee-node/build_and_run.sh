#!/bin/sh

# this are extracted from config.toml and passed to docker/bee.yaml file
export DASHBOARD_PORT=`cat ./config.toml | grep "port" | tail -1 | sed 's/["port=]//g' | tr -d '[[:space:]]'`
export MQTT_PORT=`cat ./config.toml | grep "address"| tail -1 | sed 's/["address=tcp://loclhot:]//g' | tr -d '[[:space:]]'`
export BINDING_PORT=`cat ./config.toml | grep "binding_port" | tail -1 | sed 's/["binding_port=]//g' | tr -d '[[:space:]]'`
export BIND_ADDRESS_PORT=`cat ./config.toml | grep "bind_address"| head -1 | sed 's#.*/\([^:]*\).*#\1#' | sed 's/["]//g' | tr -d '[[:space:]]'`

# docker image tag from Cargo.toml in this way the version will be in sync
IMAGE_TAG=`cat ./Cargo.toml | grep version | head -1 | sed 's/["version=]//g' | tr -d '[[:space:]]'`
export IMAGE_TAG=${IMAGE_TAG}
BUILD_BEE="cargo build --release"
BUILD_BEE_WITH_DASHBOARD="cargo build --release --features dashboard"
DOCKER_BUILD_BEE="docker build -t bee:${IMAGE_TAG} -f docker/Dockerfile ../"
RUN_DOCKER_COMPOSE="docker-compose -f ./docker/docker-compose.yml up"

# build bee node
function build_bee {
    ${BUILD_BEE}
}

# build bee node with dashboard feature
function build_bee_dashboard {
    git submodule update --init
    cd src/plugins/dashboard/frontend
    npm install
    npm run build-bee
    cd ../../../../
    ${BUILD_BEE_WITH_DASHBOARD}
}

# create a bee node docker image
function build_bee_docker {
    ${DOCKER_BUILD_BEE}
}

# run bee container
function run_bee_container {
    ${RUN_DOCKER_COMPOSE}
}

# print script help
function print_help {
    echo "Comands:"
    echo "  build"
    echo "  run"
    echo "Run './build_and_run.sh COMMAND -h' for more information on a command."
}
# print build command options
function print_build_arg {
    echo "Options:"
    echo "  bee           -> build bee node"
    echo "  bee-dashboard -> build bee node with dashboard feature"
    echo "  docker        -> create bee node docker image"
}
# print run command options
function print_run_arg {
    echo "Options:"
    echo "  bee-image -> run a bee node docker container instance"
}

# if no argument is given print the help
if [ $# -eq 0 ]; then
    print_help
    exit 0
fi
# main script logic
while [ -n "$1" ]; do
    case "$1" in
        build)
            while [ -n "$2" ]; do
                case "$2" in
                    bee)
                        build_bee
                    ;;
                    bee-dashboard)
                        build_bee_dashboard
                    ;;
                    docker)
                        build_bee_docker
                    ;;
                    -h | help)
                        print_build_arg
                    ;;
                esac
                shift
            done
        ;;
        run)
            while [ -n "$2" ]; do
                case "$2" in
                    bee-image)
                        run_bee_container
                    ;;
                    -h | help)
                        print_run_arg
                    ;;
                esac
                shift
            done
        ;;
        -h | help)
            print_help
        ;;
    esac
    shift
done
