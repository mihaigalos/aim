default:
  @just --list

test:
    #!/bin/bash
    set -x
    cargo test

    cd test/https
    just test
