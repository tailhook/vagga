setup() {
    cd /work/tests/copy
}

@test "copy: directory" {
    vagga _build dir-copy
    run vagga test-dir
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "world file sub" ]]
    link=$(readlink .vagga/dir-copy)
    [[ $link = ".roots/dir-copy.6147e496/root" ]]
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

