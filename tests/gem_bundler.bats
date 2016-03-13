setup() {
    cd /work/tests/gem_bundler
}

@test "gem/bundler: alpine pkg" {
    run vagga _run pkg-alpine rdoc --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "4.2.0" ]]
    link=$(readlink .vagga/pkg-alpine)
    [[ $link = ".roots/pkg-alpine.fc655de4/root" ]]
}
