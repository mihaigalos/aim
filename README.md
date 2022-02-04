# aim
[![CI](https://github.com/mihaigalos/aim/actions/workflows/ci.yaml/badge.svg)](https://github.com/mihaigalos/aim/actions/workflows/ci.yaml)
[![CD](https://github.com/mihaigalos/aim/actions/workflows/cd.yaml/badge.svg)](https://github.com/mihaigalos/aim/actions/workflows/cd.yaml)
[![codecov](https://codecov.io/gh/mihaigalos/aim/branch/main/graph/badge.svg?token=CYCF96JIOH)](https://codecov.io/gh/mihaigalos/aim)
[![crates.io](https://img.shields.io/crates/d/aim.svg)](https://crates.io/crates/aim)
[![docker pulls](https://img.shields.io/docker/pulls/mihaigalos/aim)](https://hub.docker.com/r/mihaigalos/aim)
[![LoC](https://tokei.rs/b1/github/mihaigalos/aim)](https://github.com/mihaigalos/aim)

A command line download/upload tool with resume.

![resume example](screenshots/aim.gif)

### Why?
Simplicity: download or upload files depending on parameter order with default settings.

### Features
* default action implied from parameter order.
  * `aim https://domain.com/` -> Display contents.
  * `aim https://domain.com/source.file .` -> Download.
  * `aim source.file https://domain.com/source.file` -> Upload.
* support for `http(s)`, `ftp`, `ssh`.
* resumable file transfers.
* optional check of sha256 checksum:
  ```rust
  aim https://github.com/XAMPPRocky/tokei/releases/download/v12.0.4/tokei-x86_64-unknown-linux-gnu.tar.gz . 0e0f0d7139c8c7e3ff20cb243e94bc5993517d88e8be8d59129730607d5c631b
  ```
* automatic no display of progressbar if content length <1MB (easy display contents of remote).

* defaults to stdout (pipe-able) to other commands:
  ```bash
  aim https://github.com/XAMPPRocky/tokei/releases/download/v12.0.4/tokei-x86_64-unknown-linux-gnu.tar.gz | tar xvz
  aim https://www.rust-lang.org/ | htmlq --attribute href a
  ```
* use as curl alternative:
  ```bash
  aim https://raw.githubusercontent.com/mihaigalos/aim/main/README.md
  ```
* configurable indicators via [`indicatif`](https://crates.io/crates/indicatif): you can change the display template and progress chars by either setting correct environment variables or creating a `.env` file in the folder you are calling from:
  ```bash
  AIM_PROGRESSBAR_DOWNLOADED_MESSAGE="🎯 Downloaded {input} to {output}"
  AIM_PROGRESSBAR_MESSAGE_FORMAT="🎯 Transfering {url}"
  AIM_PROGRESSBAR_PROGRESS_CHARS="=>-"
  AIM_PROGRESSBAR_TEMPLATE="{msg}\n{spinner:.cyan}  {elapsed_precise} ▕{bar:.white}▏ {bytes}/{total_bytes}  {bytes_per_sec}  ETA {eta}."
  AIM_PROGRESSBAR_UPLOADED_MESSAGE="🎯 Uploaded {input} to {output}"
  ```

### Resume
Resume support for both download and upload for `http(s)`, `ftp`.

Download and upload support for `ssh`, resume (using `sftp`) under development.
Currently, only user/pass auth working for `ssh`. Key support under development.

Http upload needs a webserver implementing `PUT` ranges (or a [patched](https://github.com/arut/nginx-patches) version of `nginx`).

### Authentication

Create a file named `.netrc` with read permissions in `~` or the current folder you're running `aim` from to automate login to that endpoint:
```bash
machine mydomain.com login myuser password mypass port server_port
```
### Docker

For convenience, an alpine-based docker images for `x64` and `aarch64` are available, so arguments can be passed directly to them.

```bash
docker run --rm -it -v $(pwd):/src --user $UID:$UID mihaigalos/aim https://raw.githubusercontent.com/mihaigalos/aim/main/LICENSE.md
```
### Similar work
[`duma`](https://github.com/mattgathu/duma), [`grapple`](https://github.com/daveallie/grapple), [`rget`](https://github.com/Arcterus/rget).
