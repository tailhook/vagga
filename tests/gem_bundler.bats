setup() {
    cd /work/tests/gem_bundler
}

@test "gem/bundler: alpine pkg" {
    run vagga _run pkg-alpine rake --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "rake, version 11.1.1" ]]
    link=$(readlink .vagga/pkg-alpine)
    [[ $link = ".roots/pkg-alpine.4a10a6d6/root" ]]
}

@test "gem/bundler: alpine pkg no update gem" {
    run vagga _run pkg-alpine-no-update-gem rake --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "rake, version 11.1.1" ]]
    link=$(readlink .vagga/pkg-alpine-no-update-gem)
    [[ $link = ".roots/pkg-alpine-no-update-gem.fb04a1ed/root" ]]
}

@test "gem/bundler: ubuntu trusty pkg" {
    run vagga _run pkg-ubuntu-trusty rake --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "rake, version 11.1.1" ]]
    link=$(readlink .vagga/pkg-ubuntu-trusty)
    [[ $link = ".roots/pkg-ubuntu-trusty.eed8c6d5/root" ]]
}

@test "gem/bundler: ubuntu trusty pkg no update gem" {
    run vagga _run pkg-ubuntu-trusty-no-update-gem rake --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "rake, version 11.1.1" ]]
    link=$(readlink .vagga/pkg-ubuntu-trusty-no-update-gem)
    [[ $link = ".roots/pkg-ubuntu-trusty-no-update-gem.c2dedd7b/root" ]]
}

@test "gem/bundler: ubuntu precise pkg" {
    run vagga _run pkg-ubuntu-precise rake --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "rake, version 10.5.0" ]]
    link=$(readlink .vagga/pkg-ubuntu-precise)
    [[ $link = ".roots/pkg-ubuntu-precise.59d860d7/root" ]]
}

@test "gem/bundler: ubuntu precise pkg no update gem" {
    run vagga _run pkg-ubuntu-precise-no-update-gem rake --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "rake, version 10.5.0" ]]
    link=$(readlink .vagga/pkg-ubuntu-precise-no-update-gem)
    [[ $link = ".roots/pkg-ubuntu-precise-no-update-gem.94b30704/root" ]]
}
