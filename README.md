# shared

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

## Install

You can build it from scratch or install by cargo.

```bash
git clone https://github.com/mateolafalce/shared.git
```

```bash
cargo install --path .
```

```bash
shared
```