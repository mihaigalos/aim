#!/bin/bash

err() { echo -e "\e[1;31m${@}\e[0m" >&2; exit 1; }
ok() { echo -e "\e[1;32mOK\e[0m"; }
highlight() { echo -en "\e[1;37m${@}\e[0m"; }
function wait_docker() {
    while [ $(docker ps | grep $1 | wc -l) -eq 0 ]; do
      sleep 5
    done
    sleep 5
}


export PATH=$PATH:$(realpath ../../target/debug/)
