setup() {
    cd /work/tests/copy
}

@test "copy: directory" {
    vagga _build dir-copy
    link=$(readlink .vagga/dir-copy)
    [[ $link = ".roots/dir-copy.5d6fc0cd/root" ]]

    run vagga test-dir
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "world file sub" ]]

    run vagga _run dir-copy /var/dir/exe.sh
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = "Hello!" ]]

    [[ $(stat -c "%a" ".vagga/dir-copy/var/dir") = "775" ]]
    [[ $(stat -c "%a" ".vagga/dir-copy/var/dir/hello") = "664" ]]
    [[ $(stat -c "%a" ".vagga/dir-copy/var/dir/exe.sh") = "775" ]]
    [[ $(stat -c "%a" ".vagga/dir-copy/var/dir/subdir") = "775" ]]
    [[ $(stat -c "%a" ".vagga/dir-copy/var/dir/subdir/file") = "664" ]]
}

@test "copy: file" {
    vagga _build file-copy
    link=$(readlink .vagga/file-copy)
    [[ $link = ".roots/file-copy.ac5601b3/root" ]]

    run vagga test-file
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "data" ]]

    [[ $(stat -c "%a" ".vagga/file-copy/var/file") = "664" ]]
}

@test "copy: with umask" {
    run env RUST_LOG=info vagga _build copy-umask
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/copy-umask)
    [[ $link = ".roots/copy-umask.91be8687/root" ]]

    [[ $(stat -c "%a" ".vagga/copy-umask/dir") = "700" ]]
    [[ $(stat -c "%a" ".vagga/copy-umask/dir/hello") = "600" ]]
    [[ $(stat -c "%a" ".vagga/copy-umask/dir/exe.sh") = "700" ]]
    [[ $(stat -c "%a" ".vagga/copy-umask/dir/subdir") = "700" ]]
    [[ $(stat -c "%a" ".vagga/copy-umask/dir/subdir/file") = "600" ]]
}

@test "copy: clean _unused (non-existent)" {
    run vagga _clean --unused
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
}

@test "copy: include regex" {
    run vagga _build copy-with-include
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/copy-with-include)
    [[ $link = ".roots/copy-with-include.b601121d/root" ]]
    [[ -f ".vagga/copy-with-include/dir/hello" ]]
    [[ -d ".vagga/copy-with-include/dir/subdir" ]]
    [[ -f ".vagga/copy-with-include/dir/subdir/file" ]]
    [[ ! -f ".vagga/copy-with-include/dir/second" ]]
    [[ $(vagga _version_hash copy-with-include) = $(vagga _version_hash copy-with-include-subdir) ]]
}

@test "depends: include regex" {
    run vagga _version_hash --short depends-with-include
    printf "%s\n" "${lines[@]}"
    [[ $output = "8abc830c" ]]

    chmod 0755 dir/subdir
    run vagga _version_hash --short depends-with-include
    printf "%s\n" "${lines[@]}"
    [[ $output = "8abc830c" ]]
}
