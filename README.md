# ship

Ship: a download/upload tool. ⛵

Alternatives:
* [`duma`](https://github.com/mattgathu/duma).
* [`grapple`](https://github.com/daveallie/grapple).
* [`rget`](https://github.com/Arcterus/rget).

### Why?

Some of the alternatives I could not build from source.
Others didn't have upload support or testing.
Finally, I wanted to have some fun.

### Features

* resumable file downloads via `GET` verb (default verb if none specified).
    ![resume example](screenshots/ship.gif)
* defaults to stdout (pipe-able) to other commands:
  ```bash
  ship https://github.com/XAMPPRocky/tokei/releases/download/v12.0.4/tokei-x86_64-unknown-linux-gnu.tar.gz | tar xvz
  ```
* use as curl alternative:
  ```bash
  ship https://raw.githubusercontent.com/mihaigalos/ship/main/README.md
  ```
