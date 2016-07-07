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
    [[ $link = ".roots/dir-copy.713b32f3/root" ]]
}

@test "copy: file" {
    chmod 0644 file

    vagga _build file-copy
    run vagga test-file
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "data" ]]
    link=$(readlink .vagga/file-copy)
    [[ $link = ".roots/file-copy.29314bfa/root" ]]
}

@test "copy: clean _unused (non-existent)" {
    run vagga _clean --unused
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
}

@test "copy: include regex" {
    find dir -type d -print0 | xargs -0 chmod 0755
    find dir -type f -print0 | xargs -0 chmod 0644
    chmod 0750 dir/subdir

    run env RUST_LOG=info vagga _build copy-with-include
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/copy-with-include)
    [[ $link = ".roots/copy-with-include.624cf815/root" ]]
    [[ -f ".vagga/copy-with-include/dir/hello" ]]
    [[ -d ".vagga/copy-with-include/dir/subdir" ]]
    [[ $(stat -c "%a" ".vagga/copy-with-include/dir/subdir") = "750" ]]
    [[ -f ".vagga/copy-with-include/dir/subdir/file" ]]
    [[ ! -f ".vagga/copy-with-include/dir/second" ]]
    [[ $(vagga _version_hash copy-with-include) = $(vagga _version_hash copy-with-include-subdir) ]]
}
