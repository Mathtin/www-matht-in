# Build Setup

build-path := "./build"
dist-path := build-path / "dist/"
dist-dev-path := build-path / "dist-dev/"


# Processed paths

full-build-path := shell('realpath -m ' + build-path)
full-dist-path := shell('realpath -m ' + dist-path)
full-dist-dev-path := shell('realpath -m ' + dist-dev-path)

full-log-handler-path := shell('realpath -m ./scripts/log-handler.sh')
full-foreach-path := shell('realpath -m ./scripts/foreach.sh')


# Commands

log-handler := "sh " + full-log-handler-path
foreach := "sh " + full-foreach-path

log-cmd-prefix := "date '+%Y-%m-%d %H:%M:%S.%3N' | cut -z -c -23 && echo "
log-cmd-suffix := "\" [just]\""
log := log-cmd-prefix + log-cmd-suffix
prelog-shell := log-cmd-prefix + "-n " + log-cmd-suffix + " \"$ \""


# foreach arguments

html-css-js-filter := "\\( .html .css .js \\)"
wasm-filter := "\\( .wasm \\)"


# Find commands

foreach-file := foreach + " f"
foreach-html := foreach-file +  " \\( .html \\)"
foreach-css := foreach-file +  " \\( .css \\)"
foreach-js := foreach-file +  " \\( .js \\)"
foreach-ico := foreach-file +  " \\( .ico \\)"
foreach-wasm := foreach-file +  " \\( .wasm \\)"


build:
    @{{log}} "Building shards-browser..." && {{prelog-shell}}
    wasm-pack --verbose \
            build shards-browser --target web \
            --out-dir {{full-build-path}}/shards-browser-pkg \
            --target-dir {{full-build-path}}/shards-browser \
        2>&1 | {{log-handler}} "wasm-pack"

    @{{log}} "Preparing distibutive" && {{prelog-shell}}
    mkdir -p {{full-dist-path}}

    @{{log}} "Building front-page directory tree..." && {{prelog-shell}}
    cd front-page && {{foreach}} d mkdir -p {{full-dist-path}}/{}

    @{{log}} "Minifying front-page .(html|css|js) files..." && {{prelog-shell}}
    cd front-page && {{foreach-html}} minhtml --minify-css --minify-js -o {{full-dist-path}}/{} {}
    @{{prelog-shell}}
    cd front-page && {{foreach-css}} minhtml --minify-css -o {{full-dist-path}}/{} {}
    @{{prelog-shell}}
    cd front-page && {{foreach-js}} minhtml --minify-js -o {{full-dist-path}}/{} {}

    @{{log}} "Copying front-page resource files..." && {{prelog-shell}}
    cd front-page && {{foreach-ico}} cp {} {{full-dist-path}}/{}

    @{{log}} "Removing previous .error_pages..." && {{prelog-shell}}
    rm -vr {{full-dist-path}}/.error_pages | {{log-handler}} "rm"

    @{{log}} "Moving current error_pages..." && {{prelog-shell}}
    mv {{full-dist-path}}/error_pages {{full-dist-path}}/.error_pages

    @{{log}} "Minifying shards-browser .(html|css|js) files..." && {{prelog-shell}}
    cd {{full-build-path}}/shards-browser-pkg \
        && {{foreach-js}} minhtml -o {{full-dist-path}}/{} {}

    @{{log}} "Copying shards-browser .wasm files..." && {{prelog-shell}}
    cd {{full-build-path}}/shards-browser-pkg \
        && {{foreach-wasm}} cp {} {{full-dist-path}}/{}

    @{{log}} "Success! Distributive path:" {{full-dist-path}}

build-dev:
    @{{log}} "Building shards-browser-dev: "
    wasm-pack --verbose \
            build shards-browser --dev --target web \
            --out-dir {{full-build-path}}/shards-browser-dev-pkg \
            --target-dir {{full-build-path}}/shards-browser-dev

    @{{log}} "Preparing developer distibutive: "
    mkdir -p {{full-dist-dev-path}}

    @{{log}} "Building front-page directory tree: "
    cd front-page && {{foreach}} d mkdir -p {{full-dist-dev-path}}/{}

    @{{log}} "Copying front-page files: "
    cd front-page && {{foreach-file}} cp {} {{full-dist-dev-path}}/{}

    @{{log}} "Removing previous error_pages: "
    rm -r {{full-dist-dev-path}}/.error_pages 2> /dev/null || {{log}} skipping {{full-dist-dev-path}}/.error_pages

    @{{log}} "Moving current error_pages: "
    mv {{full-dist-dev-path}}/error_pages {{full-dist-dev-path}}/.error_pages

    @{{log}} "Copying shards-browser-dev html-css-js files: "
    cd {{full-build-path}}/shards-browser-dev-pkg \
        && {{foreach-js}} cp {} {{full-dist-dev-path}}/{}

    @{{log}} "Copying shards-browser-dev wasm files: "
    cd {{full-build-path}}/shards-browser-dev-pkg \
        && {{foreach-wasm}} cp {} {{full-dist-dev-path}}/{}

    @{{log}} "Success! Developer distributive path:" {{full-dist-dev-path}}

test:
    wasm-pack test --firefox shards-browser --target-dir {{full-build-path}}/shards-browser-test
