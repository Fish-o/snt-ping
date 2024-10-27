# Ultimate Pinger 2025 Gamer Edition 

This is a project made for pinging SNT's IPv6 addresses to draw specific pixels as quickly as possible.
It automatically synchronizes it's pixel map with the image in this repository: [https://github.com/Fish-o/snt-ping-template](https://github.com/Fish-o/snt-ping-template).

## How does one use this very cool project?

First, make sure you have the [Rust programming language](https://www.rust-lang.org/tools/install)  installed.

After installing rust and cloning this repo, run `cargo build --release` to build the project.
Then run `sudo ./target/release/SNTPing` to start pinging!\

#### But what if it crashes?!? Then it'll stop pinging!!!

You're right, this is why there is the `run-forever.sh` script.
To use it, make sure it has executable permissions (`chmod +x ./run-forever.sh`) and then run `sudo ./run-forever.sh`. You're welcome!


## But but but, I dont want it to use ALL my bandwidth!

If you have very good internet, then this should not be an issue. This project can sadly not send pings at Gbps speeds. Either way, you can still reduce your bandwith usage by editing the [`src/network.rs`](/src/network.rs) file. You should probably adjust `SLEEP_PER_CYCLE` first, since that will also reduce CPU utilization. You can also change the `nops` macro if you _really_ want to wait per pixel instead of per cycle.


## What does the current image map look like?

![alt text](https://github.com/fish-o/snt-ping-template/blob/main/map.png?raw=true)

Isn't it beutiful?