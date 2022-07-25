# aim
[![CI](https://github.com/mihaigalos/aim/actions/workflows/ci.yaml/badge.svg)](https://github.com/mihaigalos/aim/actions/workflows/ci.yaml) [![CD](https://github.com/mihaigalos/aim/actions/workflows/cd.yaml/badge.svg)](https://github.com/mihaigalos/aim/actions/workflows/cd.yaml) [![codecov](https://codecov.io/gh/mihaigalos/aim/branch/main/graph/badge.svg?token=CYCF96JIOH)](https://codecov.io/gh/mihaigalos/aim) [![crates.io](https://img.shields.io/crates/d/aim.svg)](https://crates.io/crates/aim) [![docker pulls](https://img.shields.io/docker/pulls/mihaigalos/aim)](https://hub.docker.com/r/mihaigalos/aim) [![LoC](https://tokei.rs/b1/github/mihaigalos/aim)](https://github.com/mihaigalos/aim)

A command line download/upload tool with resume.

![resume example](screenshots/aim.gif)


## Table of Contents
<!--
Generated with:
docker run -v $PWD:/app -w /app --rm -it pbzweihander/markdown-toc README.md --bullet "*" --indent 2 --min-depth 1
-->
<table>
    <tr><td width=33% valign=top>

* [❓ Why](#-why)
* [💿︎ Installation](#%EF%B8%8E-installation)
* [💡 Features](#-features)
  * [Download / Upload](#download-/-upload)
  * [Optional check of sha256](#optional-check-of-sha256)
  * [Resume](#resume)
  * [Output during downloading](#output-during-downloading)
  * [Sharing a folder](#sharing-a-folder)
  * [Indicators](#indicators)
  * [Output](#output)

</td><td width=33% valign=top>

* [🔑 Authentication](#-authentication)
  * [Basicauth in url](#basicauth-in-url)
  * [Netrc](#netrc)
  * [SSH keys](#ssh-keys)
  * [.aws folder](#.aws-folder)
* [🆕 Updating](#-updating)

</td><td width=33% valign=top>

* [🐳 Docker](#-docker)
  * [Hosting on machine A](#hosting-on-machine-a)
  * [Downloading on machine B](#downloading-on-machine-b)
* [🛠️ Similar work](#%EF%B8%8F-similar-work)

</td>
</tr>
</table>

## ❓ Why
Simplicity: download or upload files depending on parameter order with default settings.

## 💿︎ Installation

Download a release for Linux or MacOS from [releases](https://github.com/mihaigalos/aim/releases). See the [Docker](https://github.com/mihaigalos/aim#docker) section on how to run it platform-independently.

If you want to build from source, use:
```bash
cargo install aim
```

## 💡 Features

### Download / Upload
* default action implied from parameter order.
  * `aim https://domain.com/` -> Display contents.
  * `aim https://domain.com/source.file .` -> Download.
  * `aim source.file https://domain.com/destination.file` -> Upload.
* support for `http(s)`, `(s)ftp`, `ssh`, `s3` (no resume at the moment).

### Optional check of sha256
To validate that a download matches a desired checksum, just list it at the end when invoking `aim`. 
```rust
aim https://github.com/XAMPPRocky/tokei/releases/download/v12.0.4/tokei-x86_64-unknown-linux-gnu.tar.gz . 0e0f0d7139c8c7e3ff20cb243e94bc5993517d88e8be8d59129730607d5c631b
```

### Resume
Resume support for both download and upload for `http(s)`, `ftp` and `sftp`.

Download and upload support for `ssh` (no resume).

If you're hosting a http(s) server yourself, upload needs `PUT` ranges (or a [patched](https://github.com/arut/nginx-patches) version of `nginx`).

### Output during downloading

Several output formats can be specified:
* `aim source .` - downloads to the same basename as the source.
* `aim source +` - downloads to the same basename as the source and attempts to decompress. Target extensions are read and the system decompressor is called. Further info [here](https://github.com/moisutsu/melt).
* `aim source destination` - download to a new or existing file called `destination`.

----------------------------------------

### Sharing a folder
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

Moreover, since hosting is done over http, the client can even be a browser:
![hosting example](screenshots/self_hosting.png)

The server prints logs to the standard output. To colorize them, you can use [pipecolor](https://github.com/dalance/pipecolor) with the provided `.pipecolor.aim.toml` in this repo:


```bash
aim /tmp | pipecolor -c ~/.pipecolor.aim.toml
```
![hosting example logs](screenshots/self_hosting_logs.png)


### Indicators
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

----------------------------------------

### Output

Because default output is stdout, `aim` is pipe-able to other commands:
```bash
aim https://github.com/XAMPPRocky/tokei/releases/download/v12.0.4/tokei-x86_64-unknown-linux-gnu.tar.gz | tar xvz
aim https://www.rust-lang.org/ | htmlq --attribute href a
```

----------------------------------------

## 🔑 Authentication

### Basicauth in url

Just use the syntax `protocol://user:pass@server:port`. This can be used for all `http(s)`, `ftp`, `ssh` and `s3`.

Example for downloading:

```bash
aim ftp://user:pass@127.0.0.1:21/myfile .
```

### Netrc

Create a file named `.netrc` with read permissions in `~` or the current folder you're running `aim` from to automate login to that endpoint:
```bash
machine mydomain.com login myuser password mypass port server_port
```

### SSH keys

Keys that match the following patterns are automatically tried:
* id_ed25519
* id_rsa
* keys/id_ed25519
* keys/id_rsa
* ~/.ssh/id_rsa
* ~/.ssh/keys/id_ed25519

### .aws folder

Credentials for AWS interaction (i.e.: S3) are automatically read from `~/.aws/credentials`.

Alternatively, the `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` environment variables are read.

----------------------------------------

## 🆕 Updating

`aim` can self update in-place using:

```bash
aim --update
```

## 🐳 Docker

For convenience, alpine-based docker images for `aarch64` and `x64` are available, so arguments can be passed directly to them.

```bash
docker run --rm -it -v $(pwd):/src --user $UID:$UID mihaigalos/aim https://raw.githubusercontent.com/mihaigalos/aim/main/LICENSE.md
```

### Hosting on machine A
```bash
cd $(mktemp -d)
echo hello > myfile
docker run --rm -it -v $(pwd):/src --user $UID:$UID -p 8080:8080 mihaigalos/aim /src
``` 
### Downloading on machine B

Adapt IP to match that of machine `A`.

```bash
docker run --rm -it -v $(pwd):/src --user $UID:$UID mihaigalos/aim http://192.168.0.24:8080/myfile /src/myfile
```
----------------------------------------

## 🛠️ Similar work
[`duma`](https://github.com/mattgathu/duma), [`grapple`](https://github.com/daveallie/grapple), [`rget`](https://github.com/Arcterus/rget).
