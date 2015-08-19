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

@test "generic: The CacheDirs tag works" {
    run vagga _run cache_dirs echo ok
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/cache_dirs)
    [[ $link = ".roots/cache_dirs.549b79e7/root" ]]
    [[ ${lines[${#lines[@]}-1]} = "ok" ]]
}

@test "generic: The supervise command works" {
    run vagga two-lines
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/busybox)
    [[ $link = ".roots/busybox.f87ff413/root" ]]
    [[ ${lines[${#lines[@]}-3]} = "hello" ]]
    [[ ${lines[${#lines[@]}-2]} = "world" ]]
}
