#!/bin/bash

function err() {
    echo -e "\e[1;31m${@}\e[0m" >&2
    exit 1
}

function ok() {
    echo -e "\e[1;32mOK\e[0m"
}

function highlight() {
    echo -en "\e[1;37m${@}\e[0m"
}

export PATH=$PATH:$(realpath ../../target/debug/)
