<div align="center">

  <a href="https://matht.in/" style="border: 1px solid; padding: 2px">
    MATHT.IN
  </a>

</div>

## About

Repository for <a href="https://matht.in/">matht.in</a> website


## ▶️ Usage

### 🛠️ Build

#### 1. Install dependencies (toolchain)

This project uses following:

* Rust <a href="https://rustup.rs/">cargo</a>
* Make-like <a href="https://github.com/casey/just">just</a> utility
* <a href="https://rustwasm.github.io/">wasm-pack</a>
* <a href="https://github.com/wilsonzlin/minify-html/tree/master/minhtml">minhtml</a> (HTML/CSS/JS minifier)

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
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall licensed as above, without any additional terms or
conditions.
