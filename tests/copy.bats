setup() {
    cd /work/tests/copy
}

@test "copy: directory" {
    find dir -type d -print0 | xargs -0 chmod 0755
    find dir -type f -print0 | xargs -0 chmod 0644
    vagga _build dir-copy
    run vagga test-dir
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "world file sub" ]]
    link=$(readlink .vagga/dir-copy)
    [[ $link = ".roots/dir-copy.789b58e2/root" ]]
}

@test "copy: file" {
    chmod 0644 file
    vagga _build file-copy
    run vagga test-file
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "data" ]]
    link=$(readlink .vagga/file-copy)
    [[ $link = ".roots/file-copy.3cf34a1f/root" ]]
}

