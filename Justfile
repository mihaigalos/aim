default:
  @just --list

tool := "aim"
docker_container_registry := "ghcr.io"
docker_user_repo := "mihaigalos"
docker_image_version := `cat Cargo.toml | grep ^version | cut -d'=' -f 2 | sed -e 's/"//g' -e 's/ //g'`
docker_image := docker_container_registry + "/" + docker_user_repo + "/" + tool+ ":" + docker_image_version

build_docker +args="":
    docker build -t {{docker_image}} {{args}} .

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
