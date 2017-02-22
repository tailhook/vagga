setup() {
    cd /work/tests/generic
}

@test "generic: The Text tag works" {
    run vagga _run text cat /etc/shakespeare
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/text)
    [[ $link = ".roots/text.efc9a869/root" ]]
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

    run vagga _build snapshot-check-mode
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    run vagga _run snapshot-check-mode stat -c "%a %U" /home/hemingway
    printf "%s\n" "${lines[@]}"
    [[ $output = "700 hemingway" ]]
    run vagga _run snapshot-check-mode stat -c "%a %U" /home/hemingway/quote.txt
    printf "%s\n" "${lines[@]}"
    [[ $output = "644 root" ]]
}

@test "generic: Snapshot from container" {
    run vagga _run snapshot-container cat /etc/shakespeare
    printf "%s\n" "${lines[@]}"
    [[ ${lines[${#lines[@]}-2]} = "Sir, in my heart there was a kind of fighting" ]]
    [[ ${lines[${#lines[@]}-1]} = "That would not let me sleep." ]]
}

@test "generic: Overriding container volumes" {
    rm -f etc/shakespeare
    run vagga override-volumes
    printf "%s\n" "${lines[@]}"
    [[ ${lines[${#lines[@]}-1]} = "yeah" ]]
    [[ $(cat etc/shakespeare) = "yeah" ]]

    rm -f etc/shakespeare
    run vagga override-volumes-supervise
    printf "%s\n" "${lines[@]}"
    [[ ${lines[${#lines[@]}-2]} = "yeah" ]]
    [[ $(cat etc/shakespeare) = "yeah" ]]
}

@test "generic: The CacheDirs tag works" {
    run vagga _run cache_dirs echo ok
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/cache_dirs)
    [[ $link = ".roots/cache_dirs.2090f8c2/root" ]]
    [[ ${lines[${#lines[@]}-1]} = "ok" ]]
}

@test "generic: The EnsureDir tag works" {
    run vagga _run ensure_dir echo ok
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/ensure_dir)
    [[ $link = ".roots/ensure_dir.998c9d5b/root" ]]
    [[ ${lines[${#lines[@]}-1]} = "ok" ]]
    [[ -d ".vagga/ensure_dir/var/lib/mount_point/subdir" ]]
    [[ $output =~ "\"/var/lib/mount_point/subdir\" directory is in the volume: \"/var/lib/mount_point\"" ]]
    [[ ! $output =~ "\"/var/lib/mount_point\" directory is in the volume: '/var/lib/mount_point'" ]]
}

@test "generic: Remove step" {
    run vagga _build remove
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/remove)
    [[ $link = ".roots/remove.2257142d/root" ]]

    [[ $(ls -1 .vagga/remove/opt/ | wc -l) = "0" ]]
}

@test "generic: The data-dirs option works" {
    run vagga _build data-container
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/data-container)
    [[ $link = ".roots/data-container.e6da9e30/root" ]]
    [[ -d ".vagga/data-container/etc" ]]
    [[ -f ".vagga/data-container/etc/passwd" ]]
    [[ -d ".vagga/data-container/var" ]]
    [[ -d ".vagga/data-container/var/lib" ]]
    [[ -d ".vagga/data-container/var/local" ]]
    [[ -f ".vagga/data-container/var/local/hello.txt" ]]
    [[ ! -f ".vagga/data-container/var/local/bye.txt" ]]
    [[ $(ls -1 ".vagga/data-container/" | wc -l) = "2" ]]
    [[ $(ls -1 ".vagga/data-container/var" | wc -l) = "2" ]]
    [[ $(ls -1 ".vagga/data-container/var/lib" | wc -l) = "3" ]]
    [[ $(ls -1 ".vagga/data-container/var/local" | wc -l) = "1" ]]
}

@test "generic: The supervise command works" {
    run vagga two-lines
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ $link = ".roots/busybox.d304a005/root" ]]
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
    run sh -c 'vagga tagged --only first_and_third | sort'
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ ${lines[${#lines[@]}-2]} = ":)" ]]
    [[ ${lines[${#lines[@]}-1]} = "hello" ]]

    run sh -c "vagga tagged --only first_and_second | sort"
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ ${lines[${#lines[@]}-2]} = "hello" ]]
    [[ ${lines[${#lines[@]}-1]} = "world" ]]

    run sh -c "vagga tagged --only third_only | sort"
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ ${lines[${#lines[@]}-1]} = ":)" ]]
}

@test "generic: The supervice --only mixed" {
    run sh -c 'vagga tagged --only first first_and_second  | sort'
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ ${lines[${#lines[@]}-2]} = "hello" ]]
    [[ ${lines[${#lines[@]}-1]} = "world" ]]

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

@test "generic: isolated Command" {
    vagga _build busybox
    run vagga --isolate-network isolated-command
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $(echo "$output" | grep "^[0-9]*:" | wc -l) = 1 ]]
}

@test "generic: isolated _run" {
    vagga _build busybox
    run vagga --no-network _run busybox ip link
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $(echo "$output" | grep "^[0-9]*:" | wc -l) = 1 ]]
}

@test "generic: isolated Supervise" {
    vagga _build busybox
    run vagga isolated-supervise
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $(echo "$output" | grep "^[0-9]*:" | wc -l) = 1 ]]
}

@test "generic: Supervise with --isolate-network option" {
    vagga _build busybox
    run vagga --no-net not-isolated-supervise
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $(echo "$output" | grep "^[0-9]*:" | wc -l) = 1 ]]
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
    [[ $link = ".roots/vagga.557b6240/root" ]]
}

@test "generic: tar without intermediate dirs" {
    rm -rf tmp tmp.tar.gz
    mkdir -p tmp/test
    chmod -R 0775 tmp
    tar czf tmp.tar.gz tmp/test

    run vagga _build tar-no-intermediate-dir
    printf "%s\n" "${lines[@]}"
    root=".vagga/tar-no-intermediate-dir"
    link=$(readlink "${root}")
    [[ $link = ".roots/tar-no-intermediate-dir.f5b7e571/root" ]]

    [[ -d "${root}/opt/tmp/test" ]]
    [[ $(stat -c "%a" "${root}/opt") = "755" ]]
    [[ $(stat -c "%a" "${root}/opt/tmp") = "755" ]]
    [[ $(stat -c "%a" "${root}/opt/tmp/test") = "775" ]]
}

@test "generic: test system dirs" {
    rm -rf tmp tmp.tar.gz
    mkdir tmp
    tar czf tmp.tar.gz tmp

    run vagga _build sys-dirs
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/sys-dirs)
    [[ $link = ".roots/sys-dirs.e66f72fd/root" ]]
    [[ $(stat -c "%a" ".vagga/sys-dirs/dev") = "755" ]]
    [[ $(stat -c "%a" ".vagga/sys-dirs/etc") = "755" ]]
    [[ $(stat -c "%a" ".vagga/sys-dirs/proc") = "755" ]]
    [[ $(stat -c "%a" ".vagga/sys-dirs/run") = "755" ]]
    [[ $(stat -c "%a" ".vagga/sys-dirs/sys") = "755" ]]
    [[ $(stat -c "%a" ".vagga/sys-dirs/tmp") = "1777" ]]
    [[ $(stat -c "%a" ".vagga/sys-dirs/work") = "755" ]]
    [[ $(ls -1 ".vagga/sys-dirs/" | wc -l) = "7" ]]
}

@test "generic: test system dirs when building container" {
    run vagga _build build-sys-dirs
    printf "%s\n" "${lines[@]}"
    [[ $output = *"/dev "* ]]
    [[ $output = *"/proc "* ]]
    [[ $output = *"/sys "* ]]
    [[ $output = *"/run "* ]]
    [[ $output != *"/tmp "* ]]
}

@test "generic: unpack zip archive" {
    curl -o test-file.zip http://files.zerogw.com/test-files/test-file.zip
    hash=($(sha256sum test-file.zip))

    cached_file="../../tmp/cache/downloads/${hash:0:8}-test-file.zip"
    rm -f $cached_file

    run vagga _build unzip-local
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/unzip-local)
    [[ $link = ".roots/unzip-local.9579aef7/root" ]]
    [[ $(cat .vagga/unzip-local/root/test/1/dir/file.txt) = "Hello" ]]
    [[ $(cat .vagga/unzip-local/root/test/1/dir/file2.txt) = "2" ]]
    [[ -x .vagga/unzip-local/root/test/1/install.sh ]]
    [[ $(cat .vagga/unzip-local/root/test/2/dir/file.txt) = "Hello" ]]
    [[ $(cat .vagga/unzip-local/root/test/2/dir/file2.txt) = "2" ]]
    [[ -x .vagga/unzip-local/root/test/2/install.sh ]]
    [[ $(cat .vagga/unzip-local/root/test/3/dir/file.txt) = "Hello" ]]
    [[ $(cat .vagga/unzip-local/root/test/3/dir/file2.txt) = "2" ]]
    [[ -x .vagga/unzip-local/root/test/3/install.sh ]]
    [[ $(cat .vagga/unzip-local/root/test/4/file.txt) = "Hello" ]]
    [[ $(cat .vagga/unzip-local/root/test/4/file2.txt) = "2" ]]
    [[ ! -d .vagga/unzip-local/root/configs/4/dir ]]
    [[ ! -f .vagga/unzip-local/root/test/4/install.sh ]]
    [[ $(cat .vagga/unzip-local/root/test/5/file.txt) = "Hello" ]]
    [[ $(cat .vagga/unzip-local/root/test/5/file2.txt) = "2" ]]
    [[ ! -d .vagga/unzip-local/root/configs/5/dir ]]
    [[ ! -f .vagga/unzip-local/root/test/5/install.sh ]]
    [[ ! -f $cached_file ]]

    run vagga _build unzip-downloaded
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/unzip-downloaded)
    [[ $link = ".roots/unzip-downloaded.fae32fd7/root" ]]
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

@test "generic: Container volume works" {
    run vagga snoop
    printf "%s\n" "${lines[@]}"
    [[ ${lines[${#lines[@]}-2]} = "Sir, in my heart there was a kind of fighting" ]]
    [[ ${lines[${#lines[@]}-1]} = "That would not let me sleep." ]]
}

@test "generic: The vagga -m works" {
    run vagga -m hello world
    printf "%s\n" "${lines[@]}"
    [[ ${lines[${#lines[@]}-1]} = "helloworld!" ]]
}

@test "generic: Hello from fake user" {
    run vagga fake-user
    printf "%s\n" "${lines[@]}"
    [[ ${lines[${#lines[@]}-1]} = "uid=1(bin) gid=0(root)" ]]
}

@test "generic: RunAs" {
    run vagga _build run-as
    printf "%s\n" "${lines[@]}"
    link=$(readlink ".vagga/run-as")
    [[ $link = ".roots/run-as.b0a77478/root" ]]

    [[ $(cat .vagga/run-as/ids-11) = "uid=1 gid=1" ]]
    [[ $(cat .vagga/run-as/ids-10) = "uid=1 gid=0" ]]
    [[ $(cat .vagga/run-as/ids-01) = "uid=0 gid=1" ]]
    [[ $(cat .vagga/run-as/ids-00) = "uid=0 gid=0" ]]
    [[ $(cat .vagga/run-as/ids-110) = "uid=1 gid=1" ]]
    [[ $(cat .vagga/run-as/var/groups) = "groups=root 501 502" ]]
    [[ ! -O .vagga/run-as/ids-11 ]]
    [[ ! -G .vagga/run-as/ids-11 ]]
    [[ ! -O .vagga/run-as/ids-10 ]]
    [[ -G .vagga/run-as/ids-10 ]]
    [[ -O .vagga/run-as/ids-01 ]]
    [[ ! -G .vagga/run-as/ids-01 ]]
    [[ -O .vagga/run-as/ids-00 ]]
    [[ -G .vagga/run-as/ids-00 ]]
    [[ -O .vagga/run-as/ids-110 ]]
    [[ ! -G .vagga/run-as/ids-110 ]]
}

@test "generic: isolated RunAs" {
    run vagga _build isolated-run-as
    printf "%s\n" "${lines[@]}"
    root=".vagga/isolated-run-as"
    link=$(readlink "${root}")
    [[ $link = ".roots/isolated-run-as.832bc83e/root" ]]

    isolated_out=$(cat "${root}/var/ip-addr-isolated.out")
    [[ $isolated_out = *"inet 127.0.0.1/8"* ]]
    [[ $isolated_out = *"inet 127.254.254.254/8"* ]]
    [[ $(cat "${root}/var/ip-link-isolated.out" | wc -l) = 2 ]]
    host_out=$(cat "${root}/var/ip-addr.out")
    [[ $host_out = *"inet 127.0.0.1/8"* ]]
    [[ $host_out != *"inet 127.254.254.254/8"* ]]
}

@test "generic: isolated RunAs with external user" {
    run vagga _build isolated-run-as-with-external-uid
    printf "%s\n" "${lines[@]}"
    root=".vagga/isolated-run-as-with-external-uid"
    link=$(readlink "${root}")
    [[ $link = ".roots/isolated-run-as-with-external-uid.59a8d2d6/root" ]]

    [[ $(cat "${root}/var/ip-link-isolated.out" | wc -l) = 2 ]]
}

@test "generic: Tmpfs Subdirs" {
    vagga _build tmpfs-subdirs
    run vagga _run tmpfs-subdirs stat -c "%A" /tmp
    printf "%s\n", "${lines[@]}"
    [[ $output = "drwxrwxrwt" ]]
    run vagga _run tmpfs-subdirs stat -c "%A" /tmp/x
    printf "%s\n", "${lines[@]}"
    [[ $output = "drwxr-xr-x" ]]
    run vagga _run tmpfs-subdirs stat -c "%A" /tmp/y
    printf "%s\n", "${lines[@]}"
    [[ $output = "drwx------" ]]
}

@test "generic: Path precedence" {
    run vagga _run path-precedence hello
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/path-precedence)
    [[ $link = ".roots/path-precedence.e2636a55/root" ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello world!" ]]
}

@test "generic: Environ precedence" {
    run vagga _build environ
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/environ)
    [[ $link = ".roots/environ.d304a005/root" ]]

    [[ $(vagga _run environ env | grep 'EDITOR=') = "EDITOR=vi" ]]
    [[ $(vagga _run environ env | grep 'SHELL=') = "SHELL=/bin/bash" ]]

    [[ $(vagga which-editor) = "vim" ]]

    mkdir -p home
    echo 'environ: {EDITOR: nvi}' > home/.vagga.yaml
    export HOME=/work/tests/generic/home
    [[ $(vagga _run environ env | grep 'EDITOR=') = "EDITOR=nvi" ]]
    [[ $(vagga which-editor) = "nvi" ]]

    echo 'environ: {EDITOR: nvi}
site_settings:
  /work/tests/generic:
    environ: {EDITOR: elvis}' > home/.vagga.yaml
    [[ $(vagga _run environ env | grep 'EDITOR=') = "EDITOR=elvis" ]]
    [[ $(vagga which-editor) = "elvis" ]]

    echo 'environ: {EDITOR: vile}' > .vagga/settings.yaml
    [[ $(vagga _run environ env | grep 'EDITOR=') = "EDITOR=vile" ]]
    [[ $(vagga which-editor) = "vile" ]]

    [[ $(VAGGAENV_EDITOR=pico vagga which-editor) = "pico" ]]

    [[ $(VAGGAENV_EDITOR=pico EDITOR=nano vagga --use-env EDITOR which-editor) = "nano" ]]

    [[ $(VAGGAENV_EDITOR=pico EDITOR=nano vagga --use-env EDITOR -E EDITOR=emacs which-editor) = "emacs" ]]

    rm -rf home
    rm -f .vagga/settings.yaml
}

@test "generic: Argument parsing for supervise" {
    run sh -c 'vagga args -Fhello --second "world"  | sort'
    printf "%s\n" "${lines[@]}"
    [[ ${lines[${#lines[@]}-2]} = "hello" ]]
    [[ ${lines[${#lines[@]}-1]} = "world" ]]

    run sh -c 'vagga args --first=x --second="y" | sort'
    printf "%s\n" "${lines[@]}"
    [[ ${lines[${#lines[@]}-2]} = "x" ]]
    [[ ${lines[${#lines[@]}-1]} = "y" ]]
}

@test "generic: Argument parsing for normal command" {
    run vagga cmdargs -vvvv --verbose
    printf "%s\n" "${lines[@]}"
    # ensure arguments is not passed directly
    [[ ${lines[${#lines[@]}-2]} = "Args:" ]]
    [[ ${lines[${#lines[@]}-1]} = "Verbosity: 5" ]]
}

@test "generic: Help with 'options'" {
    run vagga cmdargs --help
    printf "%s\n" "${lines[@]}"
    [[ "$status" -eq 0 ]]

    run vagga args --help
    printf "%s\n" "${lines[@]}"
    [[ "$status" -eq 0 ]]
}

@test "generic: Bad arguments for command with 'options'" {
    run vagga args --bad-arg
    printf "%s\n" "${lines[@]}"
    [[ "$status" -eq 121 ]]
    [[ ${lines[${#lines[@]}-2]} = "Unknown flag: '--bad-arg'" ]]
    [[ ${lines[${#lines[@]}-1]} = "Usage: vagga args [options]" ]]

    run vagga cmdargs --bad-arg
    printf "%s\n" "${lines[@]}"
    [[ "$status" -eq 121 ]]
    [[ ${lines[${#lines[@]}-1]} = "Usage: vagga cmdargs [options]" ]]

    run vagga args extra-arg
    printf "%s\n" "${lines[@]}"
    [[ "$status" -eq 121 ]]
    [[ ${lines[${#lines[@]}-1]} = "Usage: vagga args [options]" ]]

    run vagga cmdargs extra-arg
    printf "%s\n" "${lines[@]}"
    [[ "$status" -eq 121 ]]
    [[ ${lines[${#lines[@]}-1]} = "Usage: vagga cmdargs [options]" ]]
}

@test "generic: respect mount options when remounting read only" {
    mkdir -p tmp
    mount -t tmpfs -o "nodev,nosuid" tmpfs tmp
    cp vagga.yaml tmp/vagga.yaml
    cd tmp
    mkdir -p home
    run vagga check-remount-options
    cd ..
    umount tmp
    rm -rf tmp
    printf "%s\n" "${lines[@]}"
    [[ $status = 1 ]]
    # check there is no "Failed to remount readonly root" warning
    [[ $output != *"Failed to remount"* ]]
    [[ $output = *"Read-only file system"* ]]
}
