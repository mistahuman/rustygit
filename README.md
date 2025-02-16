# git-rustystats
A simple git stats tool for learning Rust

## Introduction
This project provides a simple tool to gather and display statistics from a git repository. It is designed to help you learn Rust by working on a practical project.

## Installation
To install the project, clone the repository and build it using Cargo:
```sh
git clone https://github.com/mistahuman/git-rustystats.git
cd git-rustystats
cargo build --release
```

## Getting Started
To run the project, use the following command:
```sh
cargo run
```

To create a Debian package, use:
```sh
cargo deb
```

## Usage
After building the project, you can run it to gather statistics from a git repository. For example:
```sh
./target/release/git-rustystats /path/to/git/repo
```

This will output various statistics about the repository, such as the number of commits, contributors, and more.
