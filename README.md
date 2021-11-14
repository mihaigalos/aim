# ship

⛵ Ship: a download/upload tool.

Alternatives:
[`duma`](https://github.com/mattgathu/duma), [`grapple`](https://github.com/daveallie/grapple), [`rget`](https://github.com/Arcterus/rget).

### Why?

Some of the alternatives I could not build from source.
Others didn't have upload support or testing.
Finally, I wanted to have some fun.

### Features

* resumable file transfers.
    ![resume example](screenshots/ship.gif)
* defaults to stdout (pipe-able) to other commands:
  ```bash
  ship https://github.com/XAMPPRocky/tokei/releases/download/v12.0.4/tokei-x86_64-unknown-linux-gnu.tar.gz | tar xvz
  ```
* use as curl alternative:
  ```bash
  ship https://raw.githubusercontent.com/mihaigalos/ship/main/README.md
  ```
* configurable indicators via [`indicatif`](https://crates.io/crates/indicatif): you can change the display template and progress chars by creating a `.env` file in the folder you are calling from:
  ```bash
  SHIP_PROGRESSBAR_TEMPLATE="{msg}\n{spinner:.cyan}  {elapsed_precise} ▕{bar:.white}▏ {bytes}/{total_bytes}  {bytes_per_sec}  ETA {eta}."
  SHIP_PROGRESSBAR_PROGRESS_CHARS="=>-"
  ```
