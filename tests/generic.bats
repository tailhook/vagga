setup() {
    cd /work/tests/generic
}

@test "generic: The Text tag works" {
    run vagga _run text cat /etc/shakespeare
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/text)
    [[ $link = ".roots/text.54a32625/root" ]]
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
    [[ $link = ".roots/cache_dirs.975bcc73/root" ]]
    [[ ${lines[${#lines[@]}-1]} = "ok" ]]
}

@test "generic: The EnsureDir tag works" {
    run vagga _run ensure_dir echo ok
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/ensure_dir)
    [[ $link = ".roots/ensure_dir.a1cd217a/root" ]]
    [[ ${lines[${#lines[@]}-1]} = "ok" ]]
    [[ -d ".vagga/ensure_dir/var/lib/mount_point/subdir" ]]
    [[ $output =~ "\"/var/lib/mount_point/subdir\" directory is in the volume: \"/var/lib/mount_point\"" ]]
    [[ ! $output =~ "\"/var/lib/mount_point\" directory is in the volume: '/var/lib/mount_point'" ]]
}

@test "generic: The supervise command works" {
    run vagga two-lines
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ $link = ".roots/busybox.d4736d2b/root" ]]
    [[ ${lines[${#lines[@]}-3]} = "hello" ]]
    [[ ${lines[${#lines[@]}-2]} = "world" ]]
}

@test "generic: The supervise fail-fast works" {
    run vagga one-kills-another
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ ${lines[${#lines[@]}-4]} = "hello" ]]
    [[ ${lines[${#lines[@]}-3]} = "hello" ]]
    [[ ${lines[${#lines[@]}-2]} = "world" ]]
}

@test "generic: The supervise --only" {
    run vagga two-lines --only first-line
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ ${lines[${#lines[@]}-2]} = "hello" ]]

    run vagga two-lines --only second-line
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ ${lines[${#lines[@]}-2]} = "world" ]]
}

@test "generic: The supervice --only with tags" {
    run vagga tagged --only first_and_third
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ ${lines[${#lines[@]}-3]} = "hello" ]]
    [[ ${lines[${#lines[@]}-2]} = ":)" ]]

    run vagga tagged --only first_and_second
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ ${lines[${#lines[@]}-3]} = "hello" ]]
    [[ ${lines[${#lines[@]}-2]} = "world" ]]

    run vagga tagged --only third_only
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ ${lines[${#lines[@]}-2]} = ":)" ]]
}

@test "generic: The supervice --only mixed" {
    run vagga tagged --only first first_and_second
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ ${lines[${#lines[@]}-3]} = "hello" ]]
    [[ ${lines[${#lines[@]}-2]} = "world" ]]

    run vagga tagged --only third first_and_second
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ ${lines[${#lines[@]}-4]} = "hello" ]]
    [[ ${lines[${#lines[@]}-3]} = "world" ]]
    [[ ${lines[${#lines[@]}-2]} = ":)" ]]
}

@test "generic: The supervise --exclude" {
    run vagga two-lines --exclude second-line
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ ${lines[${#lines[@]}-2]} = "hello" ]]

    run vagga two-lines --exclude first-line
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ ${lines[${#lines[@]}-2]} = "world" ]]
}

@test "generic: The supervice --exclude with tags" {
    run vagga tagged --exclude first_and_third
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ ${lines[${#lines[@]}-2]} = "world" ]]

    run vagga tagged --exclude first_and_second
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ ${lines[${#lines[@]}-2]} = ":)" ]]

    run vagga tagged --exclude third_only
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ ${lines[${#lines[@]}-3]} = "hello" ]]
    [[ ${lines[${#lines[@]}-2]} = "world" ]]
}

@test "generic: The supervice --exclude mixed" {
    run vagga tagged --exclude first first_and_second
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ ${lines[${#lines[@]}-2]} = ":)" ]]

    run vagga tagged --exclude first_and_third third_only
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
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
    [[ $link = ".roots/vagga.f4d65ae1/root" ]]
}

@test "generic: unpack zip archive" {
    curl -o test-file.zip http://files.zerogw.com/test-files/test-file.zip
    hash=($(sha256sum test-file.zip))

    cached_file="../../tmp/cache/downloads/${hash:0:8}-test-file.zip"
    rm -f $cached_file
    
    run vagga _build unzip-local
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/unzip-local)
    [[ $link = ".roots/unzip-local.24255fd9/root" ]]
    [[ $(cat .vagga/unzip-local/root/test/1/dir/file.txt) = "Hello" ]]
    [[ $(cat .vagga/unzip-local/root/test/1/dir/file2.txt) = "2" ]]
    [[ $(cat .vagga/unzip-local/root/test/2/dir/file.txt) = "Hello" ]]
    [[ $(cat .vagga/unzip-local/root/test/2/dir/file2.txt) = "2" ]]
    [[ $(cat .vagga/unzip-local/root/test/3/dir/file.txt) = "Hello" ]]
    [[ $(cat .vagga/unzip-local/root/test/3/dir/file2.txt) = "2" ]]
    [[ $(cat .vagga/unzip-local/root/test/4/file.txt) = "Hello" ]]
    [[ $(cat .vagga/unzip-local/root/test/4/file2.txt) = "2" ]]
    [[ ! -d .vagga/unzip-local/root/configs/4/dir ]]
    [[ $(cat .vagga/unzip-local/root/test/5/file.txt) = "Hello" ]]
    [[ $(cat .vagga/unzip-local/root/test/5/file2.txt) = "2" ]]
    [[ ! -d .vagga/unzip-local/root/configs/5/dir ]]
    [[ ! -f $cached_file ]]

    run vagga _build unzip-downloaded
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/unzip-downloaded)
    [[ $link = ".roots/unzip-downloaded.267da19f/root" ]]
    [[ $(cat .vagga/unzip-downloaded/root/test/dir/file.txt) = "Hello" ]]
    [[ $(cat .vagga/unzip-downloaded/root/test/dir/file2.txt) = "2" ]]
    [[ -f $cached_file ]]

    run vagga _build unzip-no-subdir
    printf "%s\n" "${lines[@]}"
    [[ $status = 121 ]]
    [[ $output = *'./dir" is not found in archive'* ]]
    [[ -f test-file.zip ]]

    run vagga _build unzip-mismatch-hashsum
    printf "%s\n" "${lines[@]}"
    [[ $status = 121 ]]
    [[ $output = *"Hashsum mismatch: expected 12345678 but was ${hash}"* ]]
    [[ -f test-file.zip ]]

    rm test-file.zip
}
