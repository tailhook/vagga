setup() {
    cd /work/tests/gem_bundler
}

@test "gem/bundler: alpine 3.3 pkg" {
    run vagga _run pkg-alpine-33 rdoc --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "4.2.0" ]]
    link=$(readlink .vagga/pkg-alpine-33)
    [[ $link = ".roots/pkg-alpine-33.fc655de4/root" ]]
}
