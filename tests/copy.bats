setup() {
    cd /work/tests/copy
}

@test "copy: directory" {
    run vagga _build dir-copy
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/dir-copy)
    [[ $link = ".roots/dir-copy.d9251b5e/root" ]]

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
    [[ $link = ".roots/file-copy.1d502439/root" ]]

    run vagga test-file
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "data" ]]

    [[ $(stat -c "%a" ".vagga/file-copy/var/file") = "664" ]]
}

@test "copy: non work" {
    run vagga _build copy-non-work
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/copy-non-work)
    [[ $link = ".roots/copy-non-work.7a6d8038/root" ]]

    [[ $(stat -c "%a" ".vagga/copy-non-work/dir.bak") = "775" ]]
    [[ $(stat -c "%a" ".vagga/copy-non-work/dir.bak/file") = "664" ]]
}

@test "copy: non work preserve permissions" {
    run vagga _build copy-non-work-preserve-perms
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/copy-non-work-preserve-perms)
    [[ $link = ".roots/copy-non-work-preserve-perms.37a30fe3/root" ]]

    [[ $(stat -c "%a" ".vagga/copy-non-work-preserve-perms/dir.bak") = \
        $(stat -c "%a" ".vagga/copy-non-work-preserve-perms/dir") ]]
    [[ $(stat -c "%a" ".vagga/copy-non-work-preserve-perms/dir.bak/file") = \
        $(stat -c "%a" ".vagga/copy-non-work-preserve-perms/dir/file") ]]
}

@test "copy: with umask" {
    run vagga _build copy-umask
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/copy-umask)
    [[ $link = ".roots/copy-umask.246e1800/root" ]]

    [[ $(stat -c "%a" ".vagga/copy-umask/dir") = "700" ]]
    [[ $(stat -c "%a" ".vagga/copy-umask/dir/hello") = "600" ]]
    [[ $(stat -c "%a" ".vagga/copy-umask/dir/exe.sh") = "700" ]]
    [[ $(stat -c "%a" ".vagga/copy-umask/dir/subdir") = "700" ]]
    [[ $(stat -c "%a" ".vagga/copy-umask/dir/subdir/file") = "600" ]]
}

@test "copy: preserve permissions" {
    chmod -R ug+rwX dir
    chmod -R o-rwx dir
    
    run vagga _build copy-preserve-perms
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/copy-preserve-perms)
    [[ $link = ".roots/copy-preserve-perms.bbb3d6ae/root" ]]

    [[ $(stat -c "%a" ".vagga/copy-preserve-perms/dir") = "770" ]]
    [[ $(stat -c "%a" ".vagga/copy-preserve-perms/dir/hello") = "660" ]]
    [[ $(stat -c "%a" ".vagga/copy-preserve-perms/dir/exe.sh") = "770" ]]
    [[ $(stat -c "%a" ".vagga/copy-preserve-perms/dir/subdir") = "770" ]]
    [[ $(stat -c "%a" ".vagga/copy-preserve-perms/dir/subdir/file") = "660" ]]
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
    [[ $link = ".roots/copy-with-include.dc355eb4/root" ]]
    [[ -f ".vagga/copy-with-include/dir/hello" ]]
    [[ -d ".vagga/copy-with-include/dir/subdir" ]]
    [[ -f ".vagga/copy-with-include/dir/subdir/file" ]]
    [[ ! -f ".vagga/copy-with-include/dir/second" ]]
    [[ $(vagga _version_hash copy-with-include) = $(vagga _version_hash copy-with-include-subdir) ]]
}

@test "depends: include regex" {
    run vagga _version_hash --short depends-with-include
    printf "%s\n" "${lines[@]}"
    [[ $output = "cd8d86d6" ]]

    chmod 0755 dir/subdir
    run vagga _version_hash --short depends-with-include
    printf "%s\n" "${lines[@]}"
    [[ $output = "cd8d86d6" ]]
}
