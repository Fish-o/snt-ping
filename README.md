# Ultimate Pinger 2025 Gamer Edition 

This is a project made for pinging SNT's IPv6 addresses to draw specific pixels as quickly as possible.
It automatically synchronizes it's pixel map with the image in this repository: [https://github.com/Fish-o/snt-ping-template](https://github.com/Fish-o/snt-ping-template).

## How does one use this very cool project?

First, make sure you have the [Rust programming language](https://www.rust-lang.org/tools/install)  installed.

After installing rust and cloning this repo, run `cargo build --release` to build the project.
Then run `sudo ./target/release/SNTPing` to start pinging!

## Configuration

You can configure this project to use less network capacity if you so wish. 
This can be done at in [`network.rs`](/src/network.rs)
To adjust network usage you should adjust `