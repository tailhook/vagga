setup() {
  cd /work/tests/volumes
}

@test "volumes: !CacheDir mount existing cache" {
    # clear cache directory if exists
    if [ -d /work/tmp/cache/cachedir-volume-test ]; then
        rm -r /work/tmp/cache/cachedir-volume-test
    fi

    run vagga cachedir-with-mounted-volume
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "1" ]]

    run vagga cachedir-without-mounted-volume
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "0" ]]

    link=$(readlink .vagga/cachedir)
    [[ $link = ".roots/cachedir.e5ee2922/root" ]]
}

@test "volumes: !CacheDir add files to cache" {
    # clear cache directory if exists
    if [ -d /work/tmp/cache/cachedir-volume-test-add-files ]; then
        rm -r /work/tmp/cache/cachedir-volume-test-add-files
    fi

    run vagga _run cachedir-add-files touch /var/cache/test1
    [[ $status = 0 ]]

    run vagga _run cachedir-add-files ls -1 /var/cache
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "test1" ]]

    run vagga _run cachedir-add-files touch /var/cache/test2
    [[ $status = 0 ]]

    run vagga _run cachedir-add-files ls -1 /var/cache
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-2]} = "test1" ]]
    [[ ${lines[${#lines[@]}-1]} = "test2" ]]

    link=$(readlink .vagga/cachedir-add-files)
    [[ $link = ".roots/cachedir-add-files.e8007744/root" ]]
}
