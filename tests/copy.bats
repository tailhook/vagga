setup() {
    cd /work/tests/copy
}

@test "copy: directory" {
    vagga _build dir-copy
    run vagga test-dir
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "world file" ]]
    link=$(readlink .vagga/dir-copy)
    [[ $link = ".roots/dir-copy.718bff63/root" ]]
}

@test "copy: file" {
    vagga _build file-copy
    run vagga test-file
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "data" ]]
    link=$(readlink .vagga/file-copy)
    [[ $link = ".roots/file-copy.cfbbcc99/root" ]]
}

