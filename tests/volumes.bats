setup() {
    cd /work/tests/volumes
}

@test "volumes: !CacheDir mount cache" {
    run vagga cachedir-count-files
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "1" ]]

    link=$(readlink .vagga/cachedir)
    [[ $link = ".roots/cachedir.9d075e39/root" ]]
}

@test "volumes: !CacheDir mount cache ubuntu" {
    run vagga cachedir-ubuntu-count-files
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "1" ]]

    link=$(readlink .vagga/cachedir-ubuntu)
    [[ $link = ".roots/cachedir-ubuntu.3b309058/root" ]]
}

@test "volumes: !CacheDir add files to cache" {
    # clear cache directory if exists
    if [ -d /work/tmp/cache/cachedir-volume-test-add-files ]; then
        rm -r /work/tmp/cache/cachedir-volume-test-add-files
    fi

    run vagga _run cachedir-add-files touch /mnt/cache/test1
    [[ $status = 0 ]]

    run vagga _run cachedir-add-files ls -1 /mnt/cache
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "test1" ]]

    run vagga _run cachedir-add-files touch /mnt/cache/test2
    [[ $status = 0 ]]

    run vagga _run cachedir-add-files ls -1 /mnt/cache
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-2]} = "test1" ]]
    [[ ${lines[${#lines[@]}-1]} = "test2" ]]

    link=$(readlink .vagga/cachedir-add-files)
    [[ $link = ".roots/cachedir-add-files.ba2005e6/root" ]]
}

@test "volumes: !CacheDir mount empty path should fail" {
    run vagga _run cachedir-mount-empty-path date
    [[ $status = 124 ]]
    [[ ${lines[${#lines[@]}-1]} = *'mount !CacheDir: path must not be empty' ]]
}

@test "volumes: !CacheDir mount absolute path should fail" {
    run vagga _run cachedir-mount-absolute-path date
    [[ $status = 124 ]]
    [[ ${lines[${#lines[@]}-1]} = *'mount !CacheDir "/cache": path must not be absolute' ]]
}
