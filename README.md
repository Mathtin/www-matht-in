<div align="center">

  <h1><code>www-matht-in</code></h1>

  <strong><a href="https://matht.in/">matht.in</a> website</strong>

  <h3>
    <a href="https://matht.in/">Main Page</a>
  </h3>
</div>

## About

Repository for <a href="https://matht.in/">matht.in</a> website

## 🚴 Usage

### 🛠️ Build



This project uses following for build procedure:

* Rust <a href="https://rustup.rs/">cargo</a>
* Make-like <a href="https://github.com/casey/just">just</a> utility
* <a href="https://rustwasm.github.io/">wasm-pack</a>
* <a href="https://github.com/wilsonzlin/minify-html/tree/master/minhtml">minhtml</a> (HTML/CSS/JS minifier)

Run following to setup toolchain (on linux):

```
sh toolchain-setup.sh
```

Run following to build release bundle:

```
just build
```

Resulting files will be stored in `build/dist`

### 🔬 Test in Browsers

```
just test
```

## License

Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall licensed as above, without any additional terms or
conditions.
