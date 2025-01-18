build:
    wasm-pack build shards-browser --target web --out-dir ../build/shards-browser-pkg --target-dir build/shards-browser

    mkdir -p build/dist

    cd front-page && find . -type d -exec mkdir -p ../build/dist/{} \;
    cd front-page && find . -type f   \( -iname \*.html -o -iname \*.css -o -iname \*.js \) -exec minhtml --minify-css --minify-js -o ../build/dist/{} {} \;
    cd front-page && find . -type f ! \( -iname \*.html -o -iname \*.css -o -iname \*.js \) -exec cp {} ../build/dist/{} \;

    rm -r build/dist/.error_pages 2> /dev/null || echo
    mv build/dist/error_pages build/dist/.error_pages

    cd build/shards-browser-pkg && find . -type f   \( -iname \*.html -o -iname \*.css -o -iname \*.js \) -exec minhtml -o ../dist/{} {} \;
    cd build/shards-browser-pkg && find . -type f -name \*.wasm -exec cp {} ../dist/{} \;

build-dev:
    wasm-pack build shards-browser --dev --target web --out-dir ../build/shards-browser-dev-pkg --target-dir build/shards-browser-dev

    mkdir -p build/dist-dev

    cd front-page && find . -type d -exec mkdir -p ../build/dist-dev/{} \;
    cd front-page && find . -type f -exec cp {} ../build/dist-dev/{} \;

    rm -r build/dist-dev/.error_pages 2> /dev/null || echo
    mv build/dist-dev/error_pages build/dist-dev/.error_pages

    cd build/shards-browser-pkg && find . -type f -exec cp {} ../dist-dev/{} \;

test:
    wasm-pack test --firefox shards-browser --target-dir ../build/shards-browser-test
