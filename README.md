<div align="center">

  [MATHT.IN](https://matht.in/)

</div>

## About

Repository for [matht.in](https://matht.in/) website


## ▶️ Usage

### 🛠️ Build

#### 1. Install toolchain

This project uses following:

* Rust [cargo](https://www.rust-lang.org/tools/install)

Install using [link](https://www.rust-lang.org/tools/install).

#### 2. Build distribution

Run following to build web bundle:

```
cargo xtask build-web-dist
```

Resulting files will be stored in `target/web-dist`

#### 3. Quickly serve

Run following to quickly setup and run [simple-http-server](https://github.com/TheWaWaR/simple-http-server) (will bind to http://127.0.0.1:8080/):

```
cargo xtask serve-web-dist
```


## License

This work is licensed under a
[Creative Commons Attribution 4.0 International License][cc-by].

[cc-by]: http://creativecommons.org/licenses/by/4.0/

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in same
license, shall licensed as above, without any additional terms or
conditions.
