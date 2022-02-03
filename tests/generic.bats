setup() {
    load '/bats/bats-support/load.bash'
    load '/bats/bats-assert/load.bash'
    cd /work/tests/generic
}

@test "generic: The Text tag works" {
    run vagga _run text cat /etc/shakespeare
    link=$(readlink .vagga/text)
    assert_equal "$link" ".roots/text.efc9a869/root"
    assert_line "Sir, in my heart there was a kind of fighting"
    assert_line "That would not let me sleep."
}

@test "generic: Snapshot volume works" {
    run vagga _run moretext cat /etc/shakespeare
    assert_equal "$(readlink .vagga/moretext)" ".roots/moretext.efc9a869/root"
    assert_line "Sir, in my heart there was a kind of fighting"
    assert_line "That would not let me sleep."

    run vagga replace-shake
    assert_success
    assert_line "nope"

    run vagga _run moretext cat /etc/shakespeare
    assert_line "Sir, in my heart there was a kind of fighting"
    assert_line "That would not let me sleep."

    run vagga _build snapshot-check-mode
    assert_success
    run vagga _run snapshot-check-mode stat -c "%a %U" /home/hemingway
    assert_output "700 hemingway"
    run vagga _run snapshot-check-mode stat -c "%a %U" /home/hemingway/quote.txt
    assert_output "644 root"
}

@test "generic: Snapshot from container" {
    run vagga _run snapshot-container cat /etc/shakespeare
    link=$(readlink .vagga/snapshot-container)
    assert_equal "$link" ".roots/snapshot-container.d304a005/root"
    assert_line "Sir, in my heart there was a kind of fighting"
    assert_line "That would not let me sleep."
}

@test "generic: Overriding container volumes" {
    rm -f etc/shakespeare
    run vagga override-volumes
    assert_success
    assert_line "yeah"
    assert_line $(cat etc/shakespeare) "yeah"

    rm -f etc/shakespeare
    run vagga override-volumes-supervise
    assert_success
    assert_line "yeah"
    assert_line $(cat etc/shakespeare) "yeah"
}

@test "generic: The CacheDirs tag works" {
    run vagga _run cache_dirs echo "hello world"
    link=$(readlink .vagga/cache_dirs)
    assert_equal "$link" ".roots/cache_dirs.2090f8c2/root"
    assert_line "hello world"
}

@test "generic: The EnsureDir tag works" {
    run vagga _run ensure_dir echo "hello world"
    link=$(readlink .vagga/ensure_dir)
    assert_equal "$link" ".roots/ensure_dir.998c9d5b/root"
    assert_line "hello world"
    assert [ -d ".vagga/ensure_dir/var/lib/mount_point/subdir" ]
    assert_output -p "\"/var/lib/mount_point/subdir\" directory is in the volume: \"/var/lib/mount_point\""
    refute_output -p "\"/var/lib/mount_point\" directory is in the volume"
}

@test "generic: Remove step" {
    run vagga _build remove
    link=$(readlink .vagga/remove)
    assert_equal "$link" ".roots/remove.2257142d/root"

    assert_equal "$(ls -1 .vagga/remove/opt/ | wc -l)" "0"
}

@test "generic: data-dirs alpine" {
    run vagga _build data-container-alpine
    container_path=.vagga/data-container-alpine
    link=$(readlink $container_path)
    assert_equal "$link" ".roots/data-container-alpine.e6da9e30/root"
    assert [ -d "$container_path/etc" ]
    assert [ -f "$container_path/etc/passwd" ]
    assert [ -d "$container_path/var" ]
    assert [ -d "$container_path/var/lib" ]
    assert [ -d "$container_path/var/local" ]
    assert [ -f "$container_path/var/local/hello.txt" ]
    assert [ ! -f "$container_path/var/local/bye.txt" ]
    assert_equal "$(ls -1 "$container_path/" | wc -l)" "2"
    assert_equal "$(ls -1 "$container_path/var" | wc -l)" "2"
    assert_equal "$(ls -1 "$container_path/var/lib" | wc -l)" "3"
    assert_equal "$(ls -1 "$container_path/var/local" | wc -l)" "1"
}

@test "generic: data-dirs ubuntu" {
    run vagga _build data-container-ubuntu
    container_path=.vagga/data-container-ubuntu
    link=$(readlink $container_path)
    assert_equal "$link" ".roots/data-container-ubuntu.ee7a0504/root"
    assert [ -d "$container_path/etc" ]
    assert [ -f "$container_path/etc/passwd" ]
    assert [ -d "$container_path/var" ]
    assert [ -d "$container_path/var/lib" ]
    assert [ -d "$container_path/var/local" ]
    assert [ -f "$container_path/var/local/hello.txt" ]
    assert [ ! -f "$container_path/var/local/bye.txt" ]
    assert_equal "$(ls -1 "$container_path/" | wc -l)" "2"
    assert_equal "$(ls -1 "$container_path/var" | wc -l)" "2"
    assert_equal "$(ls -1 "$container_path/var/lib" | wc -l)" "5"
    assert_equal "$(ls -1 "$container_path/var/local" | wc -l)" "1"
}

@test "generic: The supervise command works" {
    run vagga two-lines
    assert_success
    link=$(readlink .vagga/busybox)
    assert_equal "$link" ".roots/busybox.d304a005/root"
    assert_line "hello"
    assert_line "world"
}

@test "generic: The supervise fail-fast works" {
    run vagga one-kills-another
    assert_success
    assert_line "hello"
    assert_line "world"
    assert_line ":)"
    refute_line ":("
}

@test "generic: The supervise fail-fast with exit code" {
    run vagga one-kills-another --exit-code 1
    assert_equal "$status" 1
    assert_line "hello"
    assert_line "world"
    assert_line ":)"
    refute_line ":("
}

@test "generic: The supervise --only" {
    run vagga two-lines --only first-line
    assert_line "hello"
    refute_line "world"

    run vagga two-lines --only second-line
    refute_line "hello"
    assert_line "world"
}

@test "generic: The supervise --only with tags" {
    run vagga tagged --only first_and_third
    assert_success
    assert_line "hello"
    refute_line "world"
    assert_line ":)"

    run vagga tagged --only first_and_second
    assert_success
    assert_line "hello"
    assert_line "world"
    refute_line ":)"

    run vagga tagged --only third_only
    refute_line "hello"
    refute_line "world"
    assert_line ":)"
}

@test "generic: The supervise --only mixed" {
    run vagga tagged --only first first_and_second
    assert_success
    assert_line "hello"
    assert_line "world"
    refute_line ":)"

    run vagga tagged --only third first_and_second
    assert_success
    assert_line "hello"
    assert_line "world"
    assert_line ":)"
}

@test "generic: The supervise --exclude" {
    run vagga two-lines --exclude second-line
    assert_success
    assert_line "hello"
    refute_line "world"

    run vagga two-lines --exclude first-line
    assert_success
    refute_line "hello"
    assert_line "world"
}

@test "generic: The supervise --exclude with tags" {
    run vagga tagged --exclude first_and_third
    assert_success
    refute_line "hello"
    assert_line "world"
    refute_line ":)"

    run vagga tagged --exclude first_and_second
    assert_success
    refute_line "hello"
    refute_line "world"
    assert_line ":)"

    run vagga tagged --exclude third_only
    assert_success
    assert_line "hello"
    assert_line "world"
    refute_line ":)"
}

@test "generic: The supervise --exclude mixed" {
    run vagga tagged --exclude first first_and_second
    assert_success
    refute_line "hello"
    refute_line "world"
    assert_line ":)"

    run vagga tagged --exclude first_and_third third_only
    assert_success
    refute_line "hello"
    assert_line "world"
    refute_line ":)"
}

@test "generic: isolated Command" {
    vagga _build busybox
    run vagga isolated-command
    assert_success
    assert_equal "$(echo "$output" | grep "^[0-9]*:" | wc -l)" "1"
}

@test "generic: isolated _run" {
    vagga _build busybox
    run vagga --no-network _run busybox ip link
    assert_success
    assert_equal "$(echo "$output" | grep "^[0-9]*:" | wc -l)" "1"
}

@test "generic: isolated Supervise" {
    vagga _build busybox
    run vagga isolated-supervise
    assert_success
    assert_equal "$(echo "$output" | grep "^[0-9]*:" | wc -l)" "1"
}

@test "generic: Supervise with --isolate-network option" {
    vagga _build busybox
    run vagga --no-net not-isolated-supervise
    assert_success
    assert_equal "$(echo "$output" | grep "^[0-9]*:" | wc -l)" "1"
}

@test "generic: proxy forwards into build" {
    run env ftp_proxy=ftp://test.server vagga _build --force printenv
    assert_line "ftp_proxy=ftp://test.server"
}

@test "generic: proxy forwards into the run" {
    run env ftp_proxy=ftp://test.server vagga --no-build _run printenv env
    assert_line "ftp_proxy=ftp://test.server"
}

@test "generic: check for environment variable name validity" {
    run vagga -e key=value printenv
    assert_line 'Error propagating environment: Environment variable name (for option `-e`/`--use-env`) can'"'"'t contain equals `=` character. To set key-value pair use `-E`/`--environ` option'
}

@test "generic: unpack local tar" {
    run vagga vagga --version
    link=$(readlink .vagga/vagga)
    assert_equal "$link" ".roots/vagga.03319fd2/root"
    assert_line 'v0.4.0'
}

@test "generic: download broken file" {
    run vagga _build download-broken-file
    assert_equal "$status" 121
    assert_line -p "Hashsum mismatch:"
}

@test "generic: tar without intermediate dirs" {
    rm -rf tmp tmp.tar.gz
    mkdir -p tmp/test
    chmod -R 0775 tmp
    tar czf tmp.tar.gz tmp/test

    run vagga _build tar-no-intermediate-dir
    root=".vagga/tar-no-intermediate-dir"
    link=$(readlink "${root}")
    assert_equal "$link" ".roots/tar-no-intermediate-dir.f5b7e571/root"

    assert [ -d "${root}/opt/tmp/test" ]
    assert_equal "$(stat -c "%a" "${root}/opt")" "755"
    assert_equal "$(stat -c "%a" "${root}/opt/tmp")" "755"
    assert_equal "$(stat -c "%a" "${root}/opt/tmp/test")" "775"
}

@test "generic: test system dirs" {
    rm -rf tmp tmp.tar.gz
    mkdir tmp
    tar czf tmp.tar.gz tmp

    run vagga _build sys-dirs
    link=$(readlink .vagga/sys-dirs)
    assert_equal "$link" ".roots/sys-dirs.e66f72fd/root"
    assert_equal "$(stat -c "%a" ".vagga/sys-dirs/dev")" "755"
    assert_equal "$(stat -c "%a" ".vagga/sys-dirs/etc")" "755"
    assert_equal "$(stat -c "%a" ".vagga/sys-dirs/proc")" "755"
    assert_equal "$(stat -c "%a" ".vagga/sys-dirs/run")" "755"
    assert_equal "$(stat -c "%a" ".vagga/sys-dirs/sys")" "755"
    assert_equal "$(stat -c "%a" ".vagga/sys-dirs/tmp")" "1777"
    assert_equal "$(stat -c "%a" ".vagga/sys-dirs/work")" "755"
    assert_equal "$(ls -1 ".vagga/sys-dirs/" | wc -l)" "7"
}

@test "generic: test system dirs when building container" {
    run vagga _build build-sys-dirs
    assert_output -p "/dev "
    assert_output -p "/proc "
    assert_output -p "/sys "
    assert_output -p "/run "
    refute_output -p "/tmp "
}

@test "generic: unpack zip archive" {
    curl -o test-file.zip http://files.zerogw.com/test-files/test-file.zip
    hash=($(sha256sum test-file.zip))

    cached_file="../../tmp/cache/downloads/${hash:0:8}-test-file.zip"
    rm -f $cached_file

    run vagga _build unzip-local
    link=$(readlink .vagga/unzip-local)
    assert_equal "$link" ".roots/unzip-local.9579aef7/root"
    assert_equal "$(cat .vagga/unzip-local/root/test/1/dir/file.txt)" "Hello"
    assert_equal "$(cat .vagga/unzip-local/root/test/1/dir/file2.txt)" "2"
    assert [ -x .vagga/unzip-local/root/test/1/install.sh ]
    assert_equal "$(cat .vagga/unzip-local/root/test/2/dir/file.txt)" "Hello"
    assert_equal "$(cat .vagga/unzip-local/root/test/2/dir/file2.txt)" "2"
    assert [ -x .vagga/unzip-local/root/test/2/install.sh ]
    assert_equal "$(cat .vagga/unzip-local/root/test/3/dir/file.txt)" "Hello"
    assert_equal "$(cat .vagga/unzip-local/root/test/3/dir/file2.txt)" "2"
    assert [ -x .vagga/unzip-local/root/test/3/install.sh ]
    assert_equal "$(cat .vagga/unzip-local/root/test/4/file.txt)" "Hello"
    assert_equal "$(cat .vagga/unzip-local/root/test/4/file2.txt)" "2"
    assert [ ! -d .vagga/unzip-local/root/configs/4/dir ]
    assert [ ! -f .vagga/unzip-local/root/test/4/install.sh ]
    assert_equal "$(cat .vagga/unzip-local/root/test/5/file.txt)" "Hello"
    assert_equal "$(cat .vagga/unzip-local/root/test/5/file2.txt)" "2"
    assert [ ! -d .vagga/unzip-local/root/configs/5/dir ]
    assert [ ! -f .vagga/unzip-local/root/test/5/install.sh ]
    assert [ ! -f $cached_file ]

    run vagga _build unzip-downloaded
    link=$(readlink .vagga/unzip-downloaded)
    assert_equal "$link" ".roots/unzip-downloaded.fae32fd7/root"
    assert_equal "$(cat .vagga/unzip-downloaded/root/test/dir/file.txt)" "Hello"
    assert_equal "$(cat .vagga/unzip-downloaded/root/test/dir/file2.txt)" "2"
    assert [ -f $cached_file ]

    run vagga _build unzip-no-subdir
    assert_equal "$status" 121
    assert_line -p './dir" is not found in archive'
    assert [ -f test-file.zip ]

    run vagga _build unzip-mismatch-hashsum
    assert_equal "$status" 121
    assert_line -p "Hashsum mismatch: expected 12345678 but was ${hash}"
    assert [ -f test-file.zip ]

    rm test-file.zip
}

@test "generic: Container volume works" {
    run vagga snoop
    assert_success
    assert_line "Sir, in my heart there was a kind of fighting"
    assert_line "That would not let me sleep."
}

@test "generic: The vagga -m works" {
    run vagga -m hello world
    assert_success
    assert_line "helloworld!"
}

@test "generic: Hello from fake user" {
    run vagga fake-user
    assert_success
    assert_line "uid=1(bin) gid=0(root)"
}

@test "generic: RunAs" {
    run vagga _build run-as
    link=$(readlink ".vagga/run-as")
    assert_equal "$link" ".roots/run-as.b0a77478/root"

    assert_equal "$(cat .vagga/run-as/ids-11)" "uid=1 gid=1"
    assert_equal "$(cat .vagga/run-as/ids-10)" "uid=1 gid=0"
    assert_equal "$(cat .vagga/run-as/ids-01)" "uid=0 gid=1"
    assert_equal "$(cat .vagga/run-as/ids-00)" "uid=0 gid=0"
    assert_equal "$(cat .vagga/run-as/ids-110)" "uid=1 gid=1"
    assert_equal "$(cat .vagga/run-as/var/groups)" "groups=root 501 502"
    assert [ ! -O .vagga/run-as/ids-11 ]
    assert [ ! -G .vagga/run-as/ids-11 ]
    assert [ ! -O .vagga/run-as/ids-10 ]
    assert [ -G .vagga/run-as/ids-10 ]
    assert [ -O .vagga/run-as/ids-01 ]
    assert [ ! -G .vagga/run-as/ids-01 ]
    assert [ -O .vagga/run-as/ids-00 ]
    assert [ -G .vagga/run-as/ids-00 ]
    assert [ -O .vagga/run-as/ids-110 ]
    assert [ ! -G .vagga/run-as/ids-110 ]
}

@test "generic: isolated RunAs" {
    run vagga _build isolated-run-as
    root=".vagga/isolated-run-as"
    link=$(readlink "${root}")
    assert_equal "$link" ".roots/isolated-run-as.832bc83e/root"

    run cat "${root}/var/ip-addr-isolated.out"
    assert_line -p "inet 127.0.0.1/8"
    assert_line -p "inet 127.254.254.254/8"
    assert_equal "$(cat "${root}/var/ip-link-isolated.out" | wc -l)" "2"

    run cat "${root}/var/ip-addr.out"
    assert_line -p "inet 127.0.0.1/8"
    refute_line -p "inet 127.254.254.254/8"
}

@test "generic: isolated RunAs with external user" {
    run vagga _build isolated-run-as-with-external-uid
    root=".vagga/isolated-run-as-with-external-uid"
    link=$(readlink "${root}")
    assert_equal "$link" ".roots/isolated-run-as-with-external-uid.59a8d2d6/root"

    assert_equal "$(cat "${root}/var/ip-link-isolated.out" | wc -l)" "2"
}

@test "generic: Tmpfs Subdirs" {
    vagga _build tmpfs-subdirs
    run vagga _run tmpfs-subdirs stat -c "%A" /tmp
    assert_output "drwxrwxrwt"
    run vagga _run tmpfs-subdirs stat -c "%A" /tmp/x
    assert_output "drwxr-xr-x"
    run vagga _run tmpfs-subdirs stat -c "%A" /tmp/y
    assert_output "drwx------"
}

@test "generic: Path precedence" {
    run vagga _run path-precedence hello
    link=$(readlink .vagga/path-precedence)
    assert_equal "$link" ".roots/path-precedence.e2636a55/root"
    assert_line "Hello world!"
}

@test "generic: Environ precedence" {
    run vagga _build environ
    link=$(readlink .vagga/environ)
    assert_equal "$link" ".roots/environ.d304a005/root"

    run vagga _run environ env
    assert_line "EDITOR=vi"
    assert_line "SHELL=/bin/bash"

    run vagga which-editor
    assert_output "vim"

    mkdir -p home
    echo 'environ: {EDITOR: nvi}' > home/.vagga.yaml
    export HOME=/work/tests/generic/home
    run vagga _run environ env
    assert_line "EDITOR=nvi"
    run vagga which-editor
    assert_output "nvi"

    echo 'environ: {EDITOR: nvi}
site_settings:
  /work/tests/generic:
    environ: {EDITOR: elvis}' > home/.vagga.yaml
    run vagga _run environ env
    assert_line "EDITOR=elvis"
    run vagga which-editor
    assert_output "elvis"

    echo 'environ: {EDITOR: vile}' > .vagga/settings.yaml
    run vagga _run environ env
    assert_line "EDITOR=vile"
    run vagga which-editor
    assert_output "vile"

    run env VAGGAENV_EDITOR=pico vagga which-editor
    assert_output "pico"

    run env VAGGAENV_EDITOR=pico EDITOR=nano vagga --use-env EDITOR which-editor
    assert_output "nano"

    run env VAGGAENV_EDITOR=pico EDITOR=nano vagga --use-env EDITOR -E EDITOR=emacs which-editor
    assert_output "emacs"

    rm -rf home
    rm -f .vagga/settings.yaml
}

@test "generic: Argument parsing for supervise" {
    run vagga args -Fhello --second "world"
    assert_success
    assert_line "hello"
    assert_line "world"

    run vagga args --first=xxx --second="yyy"
    assert_success
    assert_line "xxx"
    assert_line "yyy"
}

@test "generic: Argument parsing for normal command" {
    run vagga cmdargs -vvvv --verbose
    assert_success
    # ensure arguments is not passed directly
    assert_line "Args:"
    assert_line "Verbosity: 5"
}

@test "generic: Help with 'options'" {
    run vagga cmdargs --help
    assert_success

    run vagga args --help
    assert_success
}

@test "generic: Bad arguments for command with 'options'" {
    run vagga args --bad-arg
    assert_equal "$status" 121
    assert_line "Unknown flag: '--bad-arg'"
    assert_line "Usage: vagga args [options]"

    run vagga cmdargs --bad-arg
    assert_equal "$status" 121
    assert_line "Usage: vagga cmdargs [options]"

    run vagga args extra-arg
    assert_equal "$status" 121
    assert_line "Usage: vagga args [options]"

    run vagga cmdargs extra-arg
    assert_equal "$status" 121
    assert_line "Usage: vagga cmdargs [options]"
}

@test "generic: respect mount options when remounting read only" {
    mkdir -p tmp
    mount -t tmpfs -o "nodev,nosuid" tmpfs tmp
    cp vagga.yaml tmp/vagga.yaml
    cd tmp
    mkdir -p home
    run vagga check-remount-options
    assert_equal "$status" 1
    cd ..
    umount tmp
    rm -rf tmp
    # check there is no "Failed to remount readonly root" warning
    refute_line "Failed to remount"
    assert_line -p "Read-only file system"
}

@test "generic: resolv-file-path & hosts-file-path" {
    run vagga _build resolv-conf-and-hosts
    assert_success
    link=$(readlink .vagga/resolv-conf-and-hosts)
    assert_equal "$link" ".roots/resolv-conf-and-hosts.57222830/root"
    assert_equal "$(cat .vagga/resolv-conf-and-hosts/state/resolv.conf)" ""
    assert_equal "$(cat .vagga/resolv-conf-and-hosts/state/hosts)" ""
    resolv_link=$(readlink .vagga/resolv-conf-and-hosts/etc/resolv.conf)
    hosts_link=$(readlink .vagga/resolv-conf-and-hosts/etc/hosts)
    assert_equal "$resolv_link" "/state/resolv.conf"
    assert_equal "$hosts_link" "/state/hosts"

    run vagga _run resolv-conf-and-hosts cat /state/resolv.conf
    assert_success
    assert_output "$(cat /etc/resolv.conf)"
}

@test "generic: alternate shell (bash)" {
    run vagga bash-shell
    assert_success
    assert_line '\"hello\"'
}
