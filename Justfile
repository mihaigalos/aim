default:
  @just --list --unsorted

tool := "aim"
docker_container_registry := "ghcr.io"
docker_user_repo := "mihaigalos"
docker_image_version := `cat Cargo.toml | grep ^version | cut -d'=' -f 2 | sed -e 's/"//g' -e 's/ //g'`
docker_image := docker_container_registry + "/" + docker_user_repo + "/" + tool+ ":" + docker_image_version

build:
    #!/bin/bash
    cargo build  --verbose --all || exit 1
    for d in $(find test -type d); do
        pushd $d > /dev/null
            [ -f Justfile ] && just build
        popd > /dev/null
    done

test: build
    #!/bin/bash
    cargo test  --verbose --all || exit 1

    for d in $(find test -type d); do
        pushd $d > /dev/null
            [ -f Justfile ] && just test
        popd > /dev/null
    done

setup_dockerize:
    #! /bin/bash
    sudo apt update
    sudo apt-get install -y binfmt-support qemu-user-static
    sudo apt-get install -y docker.io
    sudo usermod -aG docker $USER

    sudo apt-get install -y jq
    mkdir -p ~/.docker/cli-plugins
    BUILDX_URL=$(curl https://api.github.com/repos/docker/buildx/releases/latest |  jq  '.assets[].browser_download_url' | grep linux-arm64)
    wget $BUILDX_URL -O ~/.docker/cli-plugins/docker-build
    chmod +x ~/.docker/cli-plugins/docker-buildx

    docker buildx create --use --name mbuilder
    docker buildx inspect --bootstrap

# assumes just setup_dockerize has run at least once
dockerize_amd64 +args="":
    just _build_docker_with_buildkit "linux/amd64" {{args}}

# assumes just setup_dockerize has run at least once
dockerize_arm64 +args="":
    just _build_docker_with_buildkit "linux/arm64" {{args}}


_build_docker +args="":
    docker build -t {{docker_image}} {{args}} .

_build_docker_with_buildkit platform="linux/amd64" +args="":
    #! /bin/bash
    set -x
    platform_short=$(echo {{platform}} | cut -d '/' -f2)
    docker buildx build --platform {{platform}} {{args}} -t {{docker_image}}  --output "type=oci,dest={{tool}}_${platform_short}.tar" . && gzip {{tool}}_${platform_short}.tar
