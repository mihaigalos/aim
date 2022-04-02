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

#### Download / Upload
* default action implied from parameter order.
  * `aim https://domain.com/` -> Display contents.
  * `aim https://domain.com/source.file .` -> Download.
  * `aim source.file https://domain.com/source.file` -> Upload.
* support for `http(s)`, `ftp`, `ssh`.

#### Optional check of sha256
To validate that a download matches a desired checksum, just list it at the end when invoking `aim`. 
```rust
aim https://github.com/XAMPPRocky/tokei/releases/download/v12.0.4/tokei-x86_64-unknown-linux-gnu.tar.gz . 0e0f0d7139c8c7e3ff20cb243e94bc5993517d88e8be8d59129730607d5c631b
```

#### Resume
Resume support for both download and upload for `http(s)`, `ftp`.

Download and upload support for `ssh`, resume (using `sftp`) under development.

If you're hosting a http(s) server yourself, upload needs `PUT` ranges (or a [patched](https://github.com/arut/nginx-patches) version of `nginx`).

#### Sharing a folder
`aim` can serve a folder over `http` on one device so that you can download it on another. By default, the serving port is `8080` or the next free port.

`Machine A`
```bash
aim . # to serve current folder
```

`Machine B`
```bash
aim http://ip_of_Machine_A:8080 # list contents
aim http://ip_of_Machine_A:8080/file . # download
```

#### Indicators
By default, a progressbar is displayed when up/downloading. The indicators can be configured via the internally used [`indicatif`](https://crates.io/crates/indicatif) package.

You can change the display template and progress chars by either setting correct environment variables or creating a `.env` file in the folder you are calling from:
```bash
AIM_PROGRESSBAR_DOWNLOADED_MESSAGE="🎯 Downloaded {input} to {output}"
AIM_PROGRESSBAR_MESSAGE_FORMAT="🎯 Transfering {url}"
AIM_PROGRESSBAR_PROGRESS_CHARS="=>-"
AIM_PROGRESSBAR_TEMPLATE="{msg}\n{spinner:.cyan}  {elapsed_precise} ▕{bar:.white}▏ {bytes}/{total_bytes}  {bytes_per_sec}  ETA {eta}."
AIM_PROGRESSBAR_UPLOADED_MESSAGE="🎯 Uploaded {input} to {output}"
```

By default, no progressbar is displayed if content length <1MB (easy display contents of remote).

#### Output

Default output is stdout (pipe-able) to other commands:
```bash
aim https://github.com/XAMPPRocky/tokei/releases/download/v12.0.4/tokei-x86_64-unknown-linux-gnu.tar.gz | tar xvz
aim https://www.rust-lang.org/ | htmlq --attribute href a
```
`aim` is therefore usable as curl alternative:
```bash
aim https://raw.githubusercontent.com/mihaigalos/aim/main/README.md
```
#### Authentication

##### Basicauth in url

Just use the syntax `protocol://user:pass@server:port`. This can be used for all `http(s)`, `ftp` and `ssh`.

Example for downloading:

```bash
aim ftp://user:pass@127.0.0.1:21/myfile .
```

##### Netrc

Create a file named `.netrc` with read permissions in `~` or the current folder you're running `aim` from to automate login to that endpoint:
```bash
machine mydomain.com login myuser password mypass port server_port
```

##### SSH keys

Keys that match the following patterns are automatically tried:
* id_ed25519
* id_rsa
* keys/id_ed25519
* keys/id_rsa
* ~/.ssh/id_rsa
* ~/.ssh/keys/id_ed25519

### Docker

For convenience, alpine-based docker images for `x64` and `aarch64` are available, so arguments can be passed directly to them.

```bash
docker run --rm -it -v $(pwd):/src --user $UID:$UID mihaigalos/aim https://raw.githubusercontent.com/mihaigalos/aim/main/LICENSE.md
```

#### Hosting on machine A
```
cd $(mktemp -d)
echo hello > myfile
docker run --rm -it -v $(pwd):/src --user $UID:$UID -p 8080:8080 mihaigalos/aim /src
``` 
#### Downloading on machine B

Adapt IP to match that of machine `A`.

```bash
docker run --rm -it -v $(pwd):/src --user $UID:$UID mihaigalos/aim http://192.168.0.24:8080/myfile /src/myfile
```

### Similar work
[`duma`](https://github.com/mattgathu/duma), [`grapple`](https://github.com/daveallie/grapple), [`rget`](https://github.com/Arcterus/rget).
