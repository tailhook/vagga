setup() {
    load '/bats/bats-support/load.bash'
    load '/bats/bats-assert/load.bash'
    cd /work/tests/copy
}

@test "copy: directory" {
    run vagga _build dir-copy
    [[ $status = 0 ]]
    link=$(readlink .vagga/dir-copy)
    [[ $link = ".roots/dir-copy.781a47c8/root" ]]

    run vagga test-dir
    [[ $status = 0 ]]
    [[ ${lines[@]} = "world file sub" ]]

    run vagga _run dir-copy /var/dir/exe.sh
    [[ $status = 0 ]]
    [[ $output = "Hello!" ]]

    [[ $(stat -c "%a" ".vagga/dir-copy/var/dir") = "775" ]]
    [[ $(stat -c "%a" ".vagga/dir-copy/var/dir/hello") = "664" ]]
    [[ $(stat -c "%a" ".vagga/dir-copy/var/dir/exe.sh") = "775" ]]
    [[ $(stat -c "%a" ".vagga/dir-copy/var/dir/subdir") = "775" ]]
    [[ $(stat -c "%a" ".vagga/dir-copy/var/dir/subdir/file") = "664" ]]
    # By default we set atime and mtime to 1970-01-01 00:00:01
    [[ $(stat -c "%X" ".vagga/dir-copy/var/dir/hello") = 1 ]]
    [[ $(stat -c "%Y" ".vagga/dir-copy/var/dir/hello") = 1 ]]
}

@test "copy: file" {
    vagga _build file-copy
    link=$(readlink .vagga/file-copy)
    [[ $link = ".roots/file-copy.7ad83313/root" ]]

    run vagga test-file
    [[ $status = 0 ]]
    [[ ${lines[@]} = "data" ]]

    [[ $(stat -c "%a" ".vagga/file-copy/var/file") = "664" ]]
}

@test "copy: non work" {
    run vagga _build copy-non-work
    [[ $status = 0 ]]
    link=$(readlink .vagga/copy-non-work)
    [[ $link = ".roots/copy-non-work.5d99d983/root" ]]

    [[ $(stat -c "%a" ".vagga/copy-non-work/dir.bak") = "775" ]]
    [[ $(stat -c "%a" ".vagga/copy-non-work/dir.bak/file") = "664" ]]
}

@test "copy: non work preserve permissions" {
    run vagga _build copy-non-work-preserve-perms
    [[ $status = 0 ]]
    link=$(readlink .vagga/copy-non-work-preserve-perms)
    [[ $link = ".roots/copy-non-work-preserve-perms.0a69e70c/root" ]]

    [[ $(stat -c "%a" ".vagga/copy-non-work-preserve-perms/dir.bak") = \
        $(stat -c "%a" ".vagga/copy-non-work-preserve-perms/dir") ]]
    [[ $(stat -c "%a" ".vagga/copy-non-work-preserve-perms/dir.bak/file") = \
        $(stat -c "%a" ".vagga/copy-non-work-preserve-perms/dir/file") ]]
}

@test "copy: with umask" {
    run vagga _build copy-umask
    [[ $status = 0 ]]
    link=$(readlink .vagga/copy-umask)
    [[ $link = ".roots/copy-umask.98755219/root" ]]

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
    [[ $status = 0 ]]
    link=$(readlink .vagga/copy-preserve-perms)
    [[ $link = ".roots/copy-preserve-perms.ce6b370a/root" ]]

    [[ $(stat -c "%a" ".vagga/copy-preserve-perms/dir") = "770" ]]
    [[ $(stat -c "%a" ".vagga/copy-preserve-perms/dir/hello") = "660" ]]
    [[ $(stat -c "%a" ".vagga/copy-preserve-perms/dir/exe.sh") = "770" ]]
    [[ $(stat -c "%a" ".vagga/copy-preserve-perms/dir/subdir") = "770" ]]
    [[ $(stat -c "%a" ".vagga/copy-preserve-perms/dir/subdir/file") = "660" ]]
}

@test "copy: set owner" {
    run vagga --version

    run vagga _build copy-set-owner
    assert_success
    link=$(readlink .vagga/copy-set-owner)
    [[ $link = ".roots/copy-set-owner.e4b743de/root" ]]

    container_dir=".vagga/copy-set-owner"
    run stat -c "%u:%g" "$container_dir/dir"
    assert_output "1:2"
    run stat -c "%u:%g" "$container_dir/dir/hello"
    assert_output "1:2"
    run stat -c "%u:%g" "$container_dir/dir/subdir/file"
    assert_output "1:2"
}

@test "copy: clean _unused (non-existent)" {
    run vagga _clean --unused
    [[ $status = 0 ]]
}

@test "copy: include regex" {
    run vagga _build copy-with-include
    link=$(readlink .vagga/copy-with-include)
    [[ $link = ".roots/copy-with-include.c7333188/root" ]]
    [[ -f ".vagga/copy-with-include/dir/hello" ]]
    [[ -d ".vagga/copy-with-include/dir/subdir" ]]
    [[ -f ".vagga/copy-with-include/dir/subdir/file" ]]
    [[ ! -f ".vagga/copy-with-include/dir/second" ]]
    [[ $(vagga _version_hash copy-with-include) = $(vagga _version_hash copy-with-include-subdir) ]]
}

@test "copy: glob rules" {
    run vagga _build copy-glob-rules
    root=".vagga/copy-glob-rules"
    link=$(readlink $root)
    [[ $link = ".roots/copy-glob-rules.c7333188/root" ]]
    [[ -f "$root/dir/hello" ]]
    [[ -d "$root/dir/subdir" ]]
    [[ -f "$root/dir/subdir/file" ]]
    [[ ! -f "$root/dir/subdir/hello" ]]
    [[ ! -f "$root/dir/second" ]]
}

@test "copy: glob rules with inverse" {
    run vagga _build copy-glob-rules-inverse
    root=".vagga/copy-glob-rules-inverse"
    link=$(readlink $root)
    [[ $link = ".roots/copy-glob-rules-inverse.981e78a1/root" ]]
    [[ -f "$root/dir/hello" ]]
    [[ ! -d "$root/dir/subdir" ]]
    [[ ! -f "$root/dir/second" ]]
}

@test "copy: glob no include rules" {
    run vagga _build copy-glob-no-include-rules
    root=".vagga/copy-glob-no-include-rules"
    link=$(readlink $root)
    [[ $link = ".roots/copy-glob-no-include-rules.ee039a4f/root" ]]
    [[ -d "$root/dir" ]]
    [[ ! -f "$root/dir/exe.sh" ]]
    [[ ! -f "$root/dir/hello" ]]
    [[ ! -f "$root/dir/second" ]]
    [[ ! -d "$root/dir/subdir" ]]
    [[ $output = *"You didn't add any include rules"* ]]
}

@test "depends: include regex" {
    run vagga _version_hash --short depends-with-include
    [[ $output = "375d1004" ]]

    chmod 0755 dir/subdir
    run vagga _version_hash --short depends-with-include
    [[ $output = "375d1004" ]]
}

@test "depends: glob rules" {
    run vagga _version_hash --short depends-glob-rules
    [[ $output = "375d1004" ]]

    chmod 0755 dir/subdir
    run vagga _version_hash --short depends-glob-rules
    [[ $output = "375d1004" ]]
}

@test "copy: preserve times" {
    run vagga _build copy-preserve-times
    [[ $status = 0 ]]
    link=$(readlink .vagga/copy-preserve-times)
    [[ $link = ".roots/copy-preserve-times.6ca4b065/root" ]]
    [[ $(stat -c '%X %Y' .vagga/copy-preserve-times/dir/hello) = \
       $(stat -c '%X %Y' dir/hello) ]]
}
