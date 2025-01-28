# Variables
build-path := "./build"
dist-path := build-path / "dist/"
dist-dev-path := build-path / "dist-dev/"

log_prefix := "\"[JUST]:\" "

# Processed
full-build-path := shell('realpath -m ' + build-path)
full-dist-path := shell('realpath -m ' + dist-path)
full-dist-dev-path := shell('realpath -m ' + dist-dev-path)

# Commands
prelog := "echo -n " + log_prefix
log := "echo " + log_prefix
log-dist := log + " + " + full-dist-path
log-dist-dev := log + " + " + full-dist-dev-path

find-dir := "find . -type d"
find-file := "find . -type f"
html-css-js-filter := "\\( -iname \\*.html -o -iname \\*.css -o -iname \\*.js \\)"
find-html-css-js := find-file +  " " + html-css-js-filter
find-not-html-css-js := find-file +  " ! " + html-css-js-filter
find-wasm := find-file +  " -name \\*.wasm"


build:
    @{{prelog}} "Building shards-browser: "
    wasm-pack --verbose \
            build shards-browser --target web \
            --out-dir {{full-build-path}}/shards-browser-pkg \
            --target-dir {{full-build-path}}/shards-browser

    @{{prelog}} "Preparing distibutive: "
    mkdir -p {{full-dist-path}}

    @{{prelog}} "Building front-page directory tree: "
    cd front-page \
        && {{find-dir}} -exec {{log-dist}}/{} \; \
                        -exec mkdir -p {{full-dist-path}}/{} \;

    @{{prelog}} "Minifying front-page .(html|css|js) files: "
    cd front-page \
        && {{find-html-css-js}} -exec {{log-dist}}/{} \; \
                                -exec minhtml --minify-css --minify-js -o {{full-dist-path}}/{} {} \;

    @{{prelog}} "Copying front-page resource files: "
    cd front-page \
        && {{find-not-html-css-js}} -exec {{log-dist}}/{} \; \
                                    -exec cp {} {{full-dist-path}}/{} \;

    @{{prelog}} "Removing previous error_pages: "
    rm -r {{full-dist-path}}/.error_pages 2> /dev/null || {{log}} "skipping build/dist/.error_pages"

    @{{prelog}} "Moving current error_pages: "
    mv {{full-dist-path}}/error_pages {{full-dist-path}}/.error_pages

    @{{prelog}} "Minifying shards-browser .(html|css|js) files: "
    cd {{full-build-path}}/shards-browser-pkg \
        && {{find-html-css-js}} -exec {{log-dist}}/{} \; \
                                -exec minhtml -o {{full-dist-path}}/{} {} \;

    @{{prelog}} "Copying shards-browser .wasm files: "
    cd {{full-build-path}}/shards-browser-pkg \
        && {{find-wasm}} -exec {{log-dist}}/{} \; \
                         -exec cp {} {{full-dist-path}}/{} \;

build-dev:
    @{{prelog}} "Building shards-browser-dev: "
    wasm-pack --verbose \
            build shards-browser --dev --target web \
            --out-dir {{full-build-path}}/shards-browser-dev-pkg \
            --target-dir {{full-build-path}}/shards-browser-dev

    @{{prelog}} "Preparing developer distibutive: "
    mkdir -p {{full-dist-dev-path}}

    @{{prelog}} "Building front-page directory tree: "
    cd front-page \
        && {{find-dir}} -exec {{log-dist-dev}}/{} \; \
                        -exec mkdir -p {{full-dist-dev-path}}/{} \;

    @{{prelog}} "Copying front-page files: "
    cd front-page \
        && {{find-file}} -exec {{log-dist-dev}}/{} \; \
                         -exec cp {} ../build/dist-dev/{} \;

    @{{prelog}} "Removing previous error_pages: "
    rm -r {{full-dist-dev-path}}/.error_pages 2> /dev/null || {{log}} skipping {{full-dist-dev-path}}/.error_pages

    @{{prelog}} "Moving current error_pages: "
    mv {{full-dist-dev-path}}/error_pages {{full-dist-dev-path}}/.error_pages

    @{{prelog}} "Copying shards-browser-dev html-css-js files: "
    cd {{full-build-path}}/shards-browser-dev-pkg \
        && {{find-html-css-js}} -exec {{log-dist-dev}}/{} \; \
                                -exec cp {} {{full-dist-dev-path}}/{} \;

    @{{prelog}} "Copying shards-browser-dev wasm files: "
    cd {{full-build-path}}/shards-browser-dev-pkg \
        && {{find-wasm}} -exec {{log-dist-dev}}/{} \; \
                         -exec cp {} {{full-dist-dev-path}}/{} \;

test:
    wasm-pack test --firefox shards-browser --target-dir ../build/shards-browser-test
