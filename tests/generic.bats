setup() {
    cd /work/tests/generic
}

@test "generic: The Text tag works" {
    run vagga _run text cat /etc/shakespeare
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/text)
    [[ $link = ".roots/text.0412f7b2/root" ]]
    [[ ${lines[${#lines[@]}-2]} = "Sir, in my heart there was a kind of fighting" ]]
    [[ ${lines[${#lines[@]}-1]} = "That would not let me sleep." ]]
}

@test "generic: Snapshot volume works" {
    run vagga _run moretext cat /etc/shakespeare
    printf "%s\n" "${lines[@]}"
    [[ ${lines[${#lines[@]}-2]} = "Sir, in my heart there was a kind of fighting" ]]
    [[ ${lines[${#lines[@]}-1]} = "That would not let me sleep." ]]

    run vagga replace-shake
    printf "%s\n" "${lines[@]}"
    [[ ${lines[${#lines[@]}-1]} = "nope" ]]

    run vagga _run moretext cat /etc/shakespeare
    printf "%s\n" "${lines[@]}"
    [[ ${lines[${#lines[@]}-2]} = "Sir, in my heart there was a kind of fighting" ]]
    [[ ${lines[${#lines[@]}-1]} = "That would not let me sleep." ]]
}

@test "generic: The CacheDirs tag works" {
    run vagga _run cache_dirs echo ok
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/cache_dirs)
    [[ $link = ".roots/cache_dirs.549b79e7/root" ]]
    [[ ${lines[${#lines[@]}-1]} = "ok" ]]
}

@test "generic: The EnsureDir tag works" {
    run vagga _run ensure_dir echo ok
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/ensure_dir)
    [[ $link = ".roots/ensure_dir.d67589d0/root" ]]
    [[ ${lines[${#lines[@]}-1]} = "ok" ]]
    [[ -d ".vagga/ensure_dir/var/lib/mount_point/subdir" ]]
    [[ $output =~ "\"/var/lib/mount_point/subdir\" directory is in the volume: \"/var/lib/mount_point\"" ]]
    [[ ! $output =~ "\"/var/lib/mount_point\" directory is in the volume: '/var/lib/mount_point'" ]]
}

@test "generic: The supervise command works" {
    run vagga two-lines
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ $link = ".roots/busybox.f87ff413/root" ]]
    [[ ${lines[${#lines[@]}-3]} = "hello" ]]
    [[ ${lines[${#lines[@]}-2]} = "world" ]]
}

@test "generic: The supervise fail-fast works" {
    run vagga one-kills-another
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ $link = ".roots/busybox.f87ff413/root" ]]
    [[ ${lines[${#lines[@]}-4]} = "hello" ]]
    [[ ${lines[${#lines[@]}-3]} = "hello" ]]
    [[ ${lines[${#lines[@]}-2]} = "world" ]]
}

@test "generic: The supervise --only" {
    run vagga two-lines --only first-line
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ $link = ".roots/busybox.f87ff413/root" ]]
    [[ ${lines[${#lines[@]}-2]} = "hello" ]]

    run vagga two-lines --only second-line
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ $link = ".roots/busybox.f87ff413/root" ]]
    [[ ${lines[${#lines[@]}-2]} = "world" ]]
}

@test "generic: The supervice --only with tags" {
    run vagga tagged --only first_and_third
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ $link = ".roots/busybox.f87ff413/root" ]]
    [[ ${lines[${#lines[@]}-3]} = "hello" ]]
    [[ ${lines[${#lines[@]}-2]} = ":)" ]]

    run vagga tagged --only first_and_second
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ $link = ".roots/busybox.f87ff413/root" ]]
    [[ ${lines[${#lines[@]}-3]} = "hello" ]]
    [[ ${lines[${#lines[@]}-2]} = "world" ]]

    run vagga tagged --only third_only
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ $link = ".roots/busybox.f87ff413/root" ]]
    [[ ${lines[${#lines[@]}-2]} = ":)" ]]
}

@test "generic: The supervice --only mixed" {
    run vagga tagged --only first first_and_second
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ $link = ".roots/busybox.f87ff413/root" ]]
    [[ ${lines[${#lines[@]}-3]} = "hello" ]]
    [[ ${lines[${#lines[@]}-2]} = "world" ]]

    run vagga tagged --only third first_and_second
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ $link = ".roots/busybox.f87ff413/root" ]]
    [[ ${lines[${#lines[@]}-4]} = "hello" ]]
    [[ ${lines[${#lines[@]}-3]} = "world" ]]
    [[ ${lines[${#lines[@]}-2]} = ":)" ]]
}

@test "generic: The supervise --exclude" {
    run vagga two-lines --exclude second-line
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ $link = ".roots/busybox.f87ff413/root" ]]
    [[ ${lines[${#lines[@]}-2]} = "hello" ]]

    run vagga two-lines --exclude first-line
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ $link = ".roots/busybox.f87ff413/root" ]]
    [[ ${lines[${#lines[@]}-2]} = "world" ]]
}

@test "generic: The supervice --exclude with tags" {
    run vagga tagged --exclude first_and_third
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ $link = ".roots/busybox.f87ff413/root" ]]
    [[ ${lines[${#lines[@]}-2]} = "world" ]]

    run vagga tagged --exclude first_and_second
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ $link = ".roots/busybox.f87ff413/root" ]]
    [[ ${lines[${#lines[@]}-2]} = ":)" ]]

    run vagga tagged --exclude third_only
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ $link = ".roots/busybox.f87ff413/root" ]]
    [[ ${lines[${#lines[@]}-3]} = "hello" ]]
    [[ ${lines[${#lines[@]}-2]} = "world" ]]
}

@test "generic: The supervice --exclude mixed" {
    run vagga tagged --exclude first first_and_second
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ $link = ".roots/busybox.f87ff413/root" ]]
    [[ ${lines[${#lines[@]}-2]} = ":)" ]]

    run vagga tagged --exclude first_and_third third_only
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ $link = ".roots/busybox.f87ff413/root" ]]
    [[ ${lines[${#lines[@]}-2]} = "world" ]]
}

@test "generic: proxy forwards into build" {
    ftp_proxy=ftp://test.server run vagga _build --force printenv
    printf "%s\n" "${lines[@]}"
    [[ $(printf "%s\n" "${lines[@]}" | grep '^ftp_proxy') = \
        "ftp_proxy=ftp://test.server" ]]
}

@test "generic: proxy forwards into the run" {
    ftp_proxy=ftp://test.server run vagga --no-build _run printenv env
    printf "%s\n" "${lines[@]}"
    [[ $(printf "%s\n" "${lines[@]}" | grep '^ftp_proxy') = \
        "ftp_proxy=ftp://test.server" ]]
}

@test "generic: check for environment variable name validity" {
    run vagga -e key=value printenv
    printf "%s\n" "${lines[@]}"
    [[ ${lines[${#lines[@]}-2]} = 'Environment variable name (for option `-e`/`--use-env`) can'"'"'t contain equals `=` character. To set key-value pair use `-E`/`--environ` option' ]]
}

@test "generic: unpack local tar" {
    run vagga vagga --version
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/vagga)
    [[ ${lines[${#lines[@]}-1]} = 'v0.4.0' ]]
    [[ $link = ".roots/vagga.13a63976/root" ]]
}
