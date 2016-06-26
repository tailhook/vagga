setup() {
    cd /work/tests/gem_bundler_lock
}

teardown() {
    #if [ -d .bundle ]; then rm -r .bundle; fi
    return 0
}

@test "gem/bundler_lock: GemBundle lock" {
    run vagga _run bundle-lock rake --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "rake, version 11.1.0" ]]
    link=$(readlink .vagga/bundle-lock)
    printf "link: %s\n" "$link"
    [[ $link = ".roots/bundle-lock.e391ef51/root" ]]
}
