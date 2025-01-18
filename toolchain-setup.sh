if ! type cargo > /dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh;
fi

if ! type just > /dev/null; then
  cargo install just;
fi

if ! type wasm-pack > /dev/null; then
  cargo install wasm-pack;
fi

if ! type minhtml > /dev/null; then
  cargo install minhtml;
fi
