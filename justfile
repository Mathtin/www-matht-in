# Variables

# Commands
prelog := "echo -n \"[JUST]:\""
log := "echo \"[JUST]:\""
log-dist := log + " + build/dist/"

find-dir := "find . -type d"
find-file := "find . -type f"
html-css-js-filter := "\\( -iname \\*.html -o -iname \\*.css -o -iname \\*.js \\)"
find-html-css-js := find-file +  " " + html-css-js-filter
find-not-html-css-js := find-file +  " ! " + html-css-js-filter


build:
    @{{prelog}} "Building shards-browser: "
    wasm-pack --verbose build shards-browser --target web --out-dir ../build/shards-browser-pkg --target-dir build/shards-browser

    @{{prelog}} "Preparing distibutive: "
    mkdir -p build/dist

    @{{prelog}} "Building front-page directory tree: "
    cd front-page && {{find-dir}} -exec {{log-dist}}{} \; -exec mkdir -p ../build/dist/{} \;
    @{{prelog}} "Minifying front-page .(html|css|js) files: "
    cd front-page && {{find-html-css-js}} -exec {{log-dist}}{} \; -exec minhtml --minify-css --minify-js -o ../build/dist/{} {} \;
    @{{prelog}} "Copying front-page resource files: "
    cd front-page && {{find-not-html-css-js}} -exec {{log-dist}}{} \; -exec cp {} ../build/dist/{} \;

    @{{prelog}} "Removing previous error_pages: "
    rm -r build/dist/.error_pages 2> /dev/null || {{log}} "build/dist/.error_pages"
    @{{prelog}} "Moving current error_pages: "
    mv build/dist/error_pages build/dist/.error_pages

    @{{prelog}} "Minifying shards-browser .(html|css|js) files: "
    cd build/shards-browser-pkg && {{find-html-css-js}} -exec {{log-dist}}{} \; -exec minhtml -o ../dist/{} {} \;
    @{{prelog}} "Copying shards-browser .wasm files: "
    cd build/shards-browser-pkg && {{find-file}} -name \*.wasm -exec {{log-dist}}{} \; -exec cp {} ../dist/{} \;

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
