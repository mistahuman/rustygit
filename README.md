# rustygit
A simple git stats tool for improving Rust

## Introduction
This project provides a simple tool to gather and display statistics from a git repository. It is designed to help you learn Rust by working on a practical project.

## Installation
To install the deb released on github:
```sh
git clone https://github.com/mistahuman/rustygit.git
cd rustygit
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
For installing deb:
```sh
sudo dpkg -i rustygit-<ver>.deb
```
For launching tool:
```sh
$ rustygit --help
Program to analyze the contribution statistics of each author in a Git repository

Usage: rustygit [OPTIONS] [COMMAND]

Commands:
  changelog  Generate a changelog between two Git tags
  help       Print this message or the help of the given subcommand(s)

Options:
  -p, --path <PATH>  Path to the Git repository [default: .]
  -h, --help         Print help
  -V, --version      Print version

This will output various statistics about the repository, such as the number of commits, contributors, and more.
```
