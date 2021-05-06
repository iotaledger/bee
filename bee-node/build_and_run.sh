# docker image tag from Cargo.toml in this way the version will be in sync
IMAGE_TAG=`cat ./Cargo.toml | grep version | head -1 | sed 's/["version=]//g' | tr -d '[[:space:]]'`
export IMAGE_TAG=${IMAGE_TAG}
BUILD_BEE="cargo build --release"
BUILD_BEE_WITH_DASHBOARD="cargo build --release --features dashboard"
DOCKER_BUILD_BEE="docker build -t bee:${IMAGE_TAG} -f docker/Dockerfile ../"
RUN_DOCKER_COMPOSE="docker-compose -f ./docker/bee.yaml up"

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

function run_bee_container {
    ${RUN_DOCKER_COMPOSE}
}
function print_help {
    echo "USAGE:"
    echo
    echo "build_and_run.sh build bee -> build bee node"
    echo "build_and_run.sh build bee-dashboard -> build bee node with dashboard feature"
    echo "build_and_run.sh build docker -> create bee node docker image"
    echo "build_and_run.sh run bee-image -> create bee node docker image"
    echo "build_and_run.sh -h or help -> print help"
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