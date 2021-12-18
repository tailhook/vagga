setup() {
    cd /work/tests/gem_bundler_lock
}

teardown() {
    #if [ -d .bundle ]; then rm -r .bundle; fi
    return 0
}

@test "gem/bundler_lock: GemBundle lock" {
    run vagga _run bundle-lock rake --version
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "rake, version 11.1.0" ]]
    link=$(readlink .vagga/bundle-lock)
    [[ $link = ".roots/bundle-lock.fe593b9a/root" ]]
}
