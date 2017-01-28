setup() {
    cd /work/tests/volumes
}

@test "volumes: !CacheDir mount cache" {
    run vagga cachedir-count-files
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "1" ]]

    link=$(readlink .vagga/cachedir)
    [[ $link = ".roots/cachedir.9d075e39/root" ]]
}

@test "volumes: !CacheDir mount cache ubuntu" {
    run vagga cachedir-ubuntu-count-files
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "1" ]]

    link=$(readlink .vagga/cachedir-ubuntu)
    [[ $link = ".roots/cachedir-ubuntu.3b309058/root" ]]
}

@test "volumes: !CacheDir mount cache root" {
    run vagga _run cachedir-mount-cache-root ls /mnt/cache-root
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = *"cachedir-volume-test"* ]]

    link=$(readlink .vagga/cachedir-mount-cache-root)
    [[ $link = ".roots/cachedir-mount-cache-root.6fc500ad/root" ]]
}

@test "volumes: !CacheDir add files to cache" {
    # clear cache directory if exists
    if [ -d /work/tmp/cache/cachedir-volume-test-add-files ]; then
        rm -r /work/tmp/cache/cachedir-volume-test-add-files
    fi

    run vagga _run cachedir-add-files touch /mnt/cache/test1
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]

    run vagga _run cachedir-add-files ls -1 /mnt/cache
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "test1" ]]

    run vagga _run cachedir-add-files touch /mnt/cache/test2
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]

    run vagga _run cachedir-add-files ls -1 /mnt/cache
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-2]} = "test1" ]]
    [[ ${lines[${#lines[@]}-1]} = "test2" ]]

    link=$(readlink .vagga/cachedir-add-files)
    [[ $link = ".roots/cachedir-add-files.c12b46ac/root" ]]
}
