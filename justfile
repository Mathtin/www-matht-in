# Build Setup

build-path := "./build"
dist-path := build-path / "dist/"
dist-dev-path := build-path / "dist-dev/"


# Processed paths

full-build-path := shell('realpath -m ' + build-path)
full-dist-path := shell('realpath -m ' + dist-path)
full-dist-dev-path := shell('realpath -m ' + dist-dev-path)

full-log-handler-path := shell('realpath -m ./scripts/log-handler.sh')


# Commands

log-trap := "2>&1 | sh " + full-log-handler-path

log-cmd-prefix := "date '+%Y-%m-%d %H:%M:%S.%3N' | cut -z -c -23 && echo "
log-cmd-suffix := "\" [just]\""
log := log-cmd-prefix + log-cmd-suffix
prelog-shell := log-cmd-prefix + "-n " + log-cmd-suffix + " \"$ \""


# Find commands

find := "find ."
find-dir := find + " -type d"
find-file := find + " -type f"

find-html-css-js := find-file +  " \\( -iname \\*.html -o -iname \\*.css -o -iname \\*.js \\)"
find-js-wasm := find-file +  " \\( -iname \\*.js -o -iname \\*.wasm \\)"
find-res := find-file +  " \\( -iname \\*.ico -o -iname \\*.woff2 \\)"
find-js := find-file +  " -name \\*.js"
find-wasm := find-file +  " -name \\*.wasm"

find-log-trap := "';' -print " + log-trap + " find"


# Quick setup and run (with simple-http-server)
run: build simple-http-server
    @{{log}} "Run simple-http-server..." && {{prelog-shell}}
    simple-http-server -i -p 8080 --ip 127.0.0.1 {{full-dist-path}}

run-dev: build-dev simple-http-server
    @{{log}} "Run simple-http-server..." && {{prelog-shell}}
    simple-http-server --nocache -i -p 8080 --ip 127.0.0.1 {{full-dist-dev-path}}

simple-http-server:
    @{{log}} "Processing simple-http-server installation..." && {{prelog-shell}}
    type simple-http-server || cargo install simple-http-server


# Production builds

build-dist-dirs:
    @{{log}} "Preparing distibutive directories" && {{prelog-shell}}
    mkdir -p {{full-dist-path}}


build-shards-browser: build-dist-dirs
    @{{log}} "Building shards-browser..." && {{prelog-shell}}
    wasm-pack \
            --verbose \
            build shards-browser --target web \
            --out-dir {{full-build-path}}/shards-browser-pkg \
            --target-dir {{full-build-path}}/shards-browser \
            {{log-trap}} wasm-pack

    @{{log}} "Minifying shards-browser js files..." && {{prelog-shell}}
    cd {{full-build-path}}/shards-browser-pkg \
    && {{find-js}} -exec minhtml -o {{full-dist-path}}/{} {} \
        {{find-log-trap}} minifying ''

    @{{log}} "Copying shards-browser wasm files..." && {{prelog-shell}}
    cd {{full-build-path}}/shards-browser-pkg \
    && {{find-wasm}} -exec cp {} {{full-dist-path}}/{} \
        {{find-log-trap}} copying ''


build-front-page: build-dist-dirs
    @{{log}} "Building front-page directory tree..." && {{prelog-shell}}
    cd front-page \
    && {{find-dir}} -exec mkdir -p {{full-dist-path}}/{} \
        {{find-log-trap}} making dir {{full-dist-path}}/

    @{{log}} "Minifying front-page html, css and js files..." && {{prelog-shell}}
    cd front-page \
    && {{find-html-css-js}} -exec minhtml --minify-css --minify-js -o {{full-dist-path}}/{} {} \
        {{find-log-trap}} minifying ''

    @{{log}} "Copying front-page resource files..." && {{prelog-shell}}
    cd front-page \
    && {{find-res}} -exec cp {} {{full-dist-path}}/{} \
        {{find-log-trap}} copying ''

    @{{log}} "Removing previous .error_pages..." && {{prelog-shell}}
    rm -vr {{full-dist-path}}/.error_pages {{log-trap}} rm

    @{{log}} "Moving current error_pages..." && {{prelog-shell}}
    mv {{full-dist-path}}/error_pages {{full-dist-path}}/.error_pages {{log-trap}} mv


build: build-shards-browser build-front-page
    @{{log}} "Removing empty leftovers..." && {{prelog-shell}}
    cd {{full-dist-path}} \
    && {{find-dir}} -empty -print -delete -exec true \
        {{find-log-trap}} removing ''

    @{{log}} "Success! Distributive path:" {{full-dist-path}}


# Developer builds

build-dist-dirs-dev:
    @{{log}} "Preparing developer distibutive directories: " && {{prelog-shell}}
    mkdir -p {{full-dist-dev-path}}


build-shards-browser-dev: build-dist-dirs-dev
    @{{log}} "Building shards-browser-dev: " && {{prelog-shell}}
    wasm-pack \
            --verbose \
            build shards-browser --dev --target web \
            --out-dir {{full-build-path}}/shards-browser-dev-pkg \
            --target-dir {{full-build-path}}/shards-browser-dev \
            {{log-trap}} wasm-pack

    @{{log}} "Copying shards-browser-dev js and wasm files: " && {{prelog-shell}}
    cd {{full-build-path}}/shards-browser-dev-pkg \
    && {{find-js-wasm}} -exec cp {} {{full-dist-dev-path}}/{} \
        {{find-log-trap}} copying ''


build-front-page-dev: build-dist-dirs-dev
    @{{log}} "Building front-page directory tree: " && {{prelog-shell}}
    cd front-page \
    && {{find-dir}} -exec mkdir -p {{full-dist-dev-path}}/{} \
        {{find-log-trap}} making dir {{full-dist-path}}/

    @{{log}} "Copying front-page files: " && {{prelog-shell}}
    cd front-page \
    && {{find-file}} -exec cp {} {{full-dist-dev-path}}/{} \
        {{find-log-trap}} copying ''

    @{{log}} "Removing previous error_pages: " && {{prelog-shell}}
    rm -r {{full-dist-dev-path}}/.error_pages {{log-trap}} "rm"

    @{{log}} "Moving current error_pages: " && {{prelog-shell}}
    mv {{full-dist-dev-path}}/error_pages {{full-dist-dev-path}}/.error_pages


build-dev: build-shards-browser-dev build-front-page-dev
    @{{log}} "Success! Developer distributive path:" {{full-dist-dev-path}}


# Test builds

test:
    wasm-pack test --firefox shards-browser --target-dir {{full-build-path}}/shards-browser-test {{log-trap}} wasm-pack
