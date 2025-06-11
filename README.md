<div align="center">

  [MATHT.IN](https://matht.in/)

</div>

## About

Repository for [matht.in](https://matht.in/) website


## ▶️ Usage

### 🛠️ Build

#### 1. Install dependencies (toolchain)

This project uses following:

* Rust [cargo](https://rustup.rs/)
* Make-like [just](https://github.com/casey/just) utility
* [wasm-pack](https://rustwasm.github.io/)
* [minhtml](https://github.com/wilsonzlin/minify-html/tree/master/minhtml) (HTML/CSS/JS minifier)

Run following to setup (on linux shell):

```
sh toolchain-setup.sh
```

#### 2. Build distribution

Run following to build release bundle:

```
just build
```

Resulting files will be stored in `build/dist`

#### 3. Quickly serve

Run following to quickly setup and run [simple-http-server](https://github.com/TheWaWaR/simple-http-server) (will bind to http://127.0.0.1:8080/index.html):

```
just run
```


## Other

### Tests

To run tests in browsers (start a webserver on localhost:8000)

```
just test
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
