
# aim
[![CI](https://github.com/mihaigalos/aim/actions/workflows/ci.yaml/badge.svg)](https://github.com/mihaigalos/aim/actions/workflows/ci.yaml)
[![crates.io](https://img.shields.io/crates/d/aim.svg)](https://crates.io/crates/aim)

🎯 aim: A command line download/upload tool with resume.

![resume example](screenshots/aim.gif)

### Alternatives:
[`duma`](https://github.com/mattgathu/duma), [`grapple`](https://github.com/daveallie/grapple), [`rget`](https://github.com/Arcterus/rget).

### Why?
Simplicity: download or upload files depending on parameter order with default settings.

### Features
* default action implied from parameter order.
  * `aim https://domain.com/` -> Display contents.
  * `aim https://domain.com/source.file .` -> Download.
  * `aim source.file https://domain.com/source.file` -> Upload.

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

* Download resume:
  * [x] http
  * [x] ftp
* Upload resume:
  * [x] http implemented but experimental. You need a webserver implementing `PUT` ranges (or a [patched](https://github.com/arut/nginx-patches) version of `nginx`). `_test_aim_put_resume_binary_file` should cover this case.
  * [x] ftp

### Authentication

`.netrc` support is implemented. Create a file named `.netrc` with read permissions in `~` or the current folder you're running `aim` from to automate login to the endpoint:
```bash
machine mydomain.com login myuser password mypass port server_port
```
### Docker

For convenience, an alpine-based docker image is available, so arguments can be passed directly to it.
Future versions will support single naming for all archs via manifests.

`x64`:
```bash
docker pull mihaigalos/aim:0.0.3
docker run --rm -it -v $(pwd):/src --user $UID:$UID mihaigalos/aim:0.0.3 https://raw.githubusercontent.com/mihaigalos/aim/main/LICENSE.md
```

`arm64/aarch64`:
```bash
docker pull mihaigalos/aim-arm64:0.0.3
docker run --rm -it -v $(pwd):/src --user $UID:$UID mihaigalos/aim-arm64:0.0.3 https://raw.githubusercontent.com/mihaigalos/aim/main/LICENSE.md
```
