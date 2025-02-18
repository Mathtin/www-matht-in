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
find-js := find-file +  " -name \\*.js"
find-res := find-file +  " -name \\*.ico"
find-wasm := find-file +  " -name \\*.wasm"

find-log := "';' -print " + log-trap + " find"


build:
    @echo "Building shards-browser..." {{log-trap}} just && {{prelog-shell}}
    wasm-pack \
            --verbose \
            build shards-browser --target web \
            --out-dir {{full-build-path}}/shards-browser-pkg \
            --target-dir {{full-build-path}}/shards-browser \
            {{log-trap}} wasm-pack

    @{{log}} "Preparing distibutive" && {{prelog-shell}}
    mkdir -p {{full-dist-path}}

    @{{log}} "Building front-page directory tree..." && {{prelog-shell}}
    cd front-page \
    && {{find-dir}} -exec mkdir -p {{full-dist-path}}/{} \
        {{find-log}} making {{full-dist-path}}/

    @{{log}} "Minifying front-page html, css and js files..." && {{prelog-shell}}
    cd front-page \
    && {{find-html-css-js}} -exec minhtml --minify-css --minify-js -o {{full-dist-path}}/{} {} \
        {{find-log}} minifying ''

    @{{log}} "Copying front-page resource files..." && {{prelog-shell}}
    cd front-page \
    && {{find-res}} -exec cp {} {{full-dist-path}}/{} \
        {{find-log}} copying ''

    @{{log}} "Removing previous .error_pages..." && {{prelog-shell}}
    rm -vr {{full-dist-path}}/.error_pages {{log-trap}} rm

    @{{log}} "Moving current error_pages..." && {{prelog-shell}}
    mv {{full-dist-path}}/error_pages {{full-dist-path}}/.error_pages {{log-trap}} mv

    @{{log}} "Minifying shards-browser js files..." && {{prelog-shell}}
    cd {{full-build-path}}/shards-browser-pkg \
    && {{find-js}} -exec minhtml -o {{full-dist-path}}/{} {} \
        {{find-log}} minifying ''

    @{{log}} "Copying shards-browser wasm files..." && {{prelog-shell}}
    cd {{full-build-path}}/shards-browser-pkg \
    && {{find-wasm}} -exec cp {} {{full-dist-path}}/{} \
        {{find-log}} copying ''

    @{{log}} "Success! Distributive path:" {{full-dist-path}}

build-dev:
    @{{log}} "Building shards-browser-dev: " && {{prelog-shell}}
    wasm-pack \
            --verbose \
            build shards-browser --dev --target web \
            --out-dir {{full-build-path}}/shards-browser-dev-pkg \
            --target-dir {{full-build-path}}/shards-browser-dev \
            {{log-trap}} wasm-pack

    @{{log}} "Preparing developer distibutive: " && {{prelog-shell}}
    mkdir -p {{full-dist-dev-path}}

    @{{log}} "Building front-page directory tree: " && {{prelog-shell}}
    cd front-page \
    && {{find-dir}} -exec mkdir -p {{full-dist-dev-path}}/{} \
        {{find-log}} making dir {{full-dist-path}}/

    @{{log}} "Copying front-page files: " && {{prelog-shell}}
    cd front-page \
    && {{find-file}} -exec cp {} {{full-dist-dev-path}}/{} \
        {{find-log}} copying ''

    @{{log}} "Removing previous error_pages: " && {{prelog-shell}}
    rm -r {{full-dist-dev-path}}/.error_pages {{log-trap}} "rm"

    @{{log}} "Moving current error_pages: " && {{prelog-shell}}
    mv {{full-dist-dev-path}}/error_pages {{full-dist-dev-path}}/.error_pages

    @{{log}} "Copying shards-browser-dev js and wasm files: " && {{prelog-shell}}
    cd {{full-build-path}}/shards-browser-dev-pkg \
    && {{find-js-wasm}} -exec cp {} {{full-dist-dev-path}}/{} \
        {{find-log}} copying ''

    @{{log}} "Success! Developer distributive path:" {{full-dist-dev-path}}

test:
    wasm-pack test --firefox shards-browser --target-dir {{full-build-path}}/shards-browser-test
