
# aim
[![CI](https://github.com/mihaigalos/aim/actions/workflows/ci.yaml/badge.svg)](https://github.com/mihaigalos/aim/actions/workflows/rust.yaml)
[![crates.io](https://img.shields.io/crates/d/aim.svg)](https://crates.io/crates/aim)

🎯 aim: A command line download/upload tool with resume.

Alternatives:
[`duma`](https://github.com/mattgathu/duma), [`grapple`](https://github.com/daveallie/grapple), [`rget`](https://github.com/Arcterus/rget).

### Why?
Simplicity: a modern, simple tool for downloading/uploading with default settings.

### Features
* default action implied from parameter order.
  * `aim https://domain.com/"` -> Display contents.
  * `aim https://domain.com/source.file source.file"` -> Download.
  * `aim source.file https://domain.com/source.file"` -> Upload.

* resumable file transfers.
* automatic no display of progressbar if content length <1MB (easy display contents of remote).

  ![resume example](screenshots/aim.gif)
* defaults to stdout (pipe-able) to other commands:
  ```bash
  aim https://github.com/XAMPPRocky/tokei/releases/download/v12.0.4/tokei-x86_64-unknown-linux-gnu.tar.gz | tar xvz
  ```
* use as curl alternative:
  ```bash
  aim https://raw.githubusercontent.com/mihaigalos/aim/main/README.md
  ```
* configurable indicators via [`indicatif`](https://crates.io/crates/indicatif): you can change the display template and progress chars by either setting correct environment variables or creating a `.env` file in the folder you are calling from:
  ```bash
  AIM_PROGRESSBAR_MESSAGE_FORMAT="🎯 Transfering {url}"
  AIM_PROGRESSBAR_TEMPLATE="{msg}\n{spinner:.cyan}  {elapsed_precise} ▕{bar:.white}▏ {bytes}/{total_bytes}  {bytes_per_sec}  ETA {eta}."
  AIM_PROGRESSBAR_PROGRESS_CHARS="=>-"
  ```
* pipe-able output: `aim https://www.rust-lang.org/ | htmlq --attribute href a`

### Resume

* Download resume:
  * [x] http
  * [x] ftp
* Upload resume:
  * [x] http implemented but experimental. You need a webserver implementing `PUT` ranges (or a [patched](https://github.com/arut/nginx-patches) version of `nginx`). `_test_aim_put_resume_binary_file` should cover this case.
  * [ ] ftp
