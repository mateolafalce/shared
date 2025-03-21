<div align="center">

<img style="float: right; height: 192px; width: 192px;" src="https://github.com/mateolafalce/shared/blob/main/static/icon.png"/>

# Shared

[<img alt="crates.io" src="https://img.shields.io/crates/v/shared.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/shared)
[<img alt="github" src="https://img.shields.io/badge/github-mateolafalce/shared-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/mateolafalce/shared)

</div>

A **web solution developed in Rust** that allows one device on a shared network to **share its screen** with others. It is designed for pair programming and technical meetings within development companies, as well as for individual use The computer sharing its local screen acts as a **server** and streams a bitstream representing the pixels of the shared image to anyone who connects to the specified URL.

The program is designed so that **one screen** is shared with **multiple client devices**. There are two routes:

- At `/admin`, the user can share their screen with others.
- At `/`, users can view the screen being shared.


## Roadmap

* Features
   * [x] Share the screen
   * [ ] Optionally audio share
* OS Support (as a server)
   * [x] Linux
   * [x] Windows
   * [x] Mac

The program allows different administrators to share their screens within the same network using an algorithm that checks whether the current port (3000) is in use or not. If it is in use, it checks port 3001; if that is also in use, it checks port 3002, and so on.

## Execute

You can run the latest shader version for x86_64 on Linux devices [here](https://github.com/mateolafalce/shared/releases).


## Compile and Run

You can run shared from scratch.

```bash
git clone https://github.com/mateolafalce/shared.git
```

```bash
cargo run --release
```

## Install

```bash
cargo install shared
```

## Help

```bash
shared --help
```

```bash
./shared.AppImage --help
```

```bash
cargo run -- --help
```

Output:

```bash
Options:
  -p, --port <PORT>    The port you want the program to run on, by default 3000
  -t, --title <TITLE>  The title of the page, by default "shared"
  -h, --help           Print help
  -V, --version        Print version
```

## Custom CLI args

```bash
./shared.AppImage --title "Class: digital signatures" --port 1234
```

```bash
shared --title "Class: digital signatures" --port 1234
```

```bash
cargo run -- --title "Class: digital signatures" --port 1234
```

## Demo

![Demo](static/how_works.gif)
