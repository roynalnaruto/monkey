# Monkey
Monkey is a command-line P2P toy blockchain.

[![Build Status]][Build Link]

[Build Status]: https://github.com/roynalnaruto/monkey/workflows/Test%20Suite/badge.svg?branch=master
[Build Link]: https://github.com/roynalnaruto/monkey/actions

## Overview

In Monkey world, a block consists of a set of English words, whereby this set satisfies the protocol regulations. Players can compete with each other by coming up with such valid sets of words, and hence producing blocks for Monkey.

*Note:* This project is a toy blockchain, which I have been developing while exploring LibP2P and Async programming in Rust. There is no monetary value associated with this project.

## Demo

<p align="center"><img src="/img/demo.gif?raw=true"/></p>

## Setup

Monkey uses Rust's nightly release for its tests.

* Install nightly
```
$ rustup toolchain install nightly
```
* Compile source code and tests
```
$ cargo +nightly build
$ cargo +nightly build --tests
```
* Run test suite
```
$ cargo +nightly test
```

## Run

Monkey should be run in multiple terminal instances, where every instance represents a peer in the peer-to-peer network.

* Start monkey as the first peer
```
$ ./target/debug/monkey peer_a_db
```
* Start monkey while connecting with an existing peer (Replace `xxxxx` with peer A's listener port)
```
# ./target/debug/monkey peer_b_db /ip4/127.0.0.1/tcp/xxxxx
```
