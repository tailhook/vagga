setup() {
    cd /work/tests/npm
}

@test "npm: default pkg" {
    run vagga _run pkg resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/pkg)
    [[ $link = ".roots/pkg.e76ae0ec/root" ]]
}

@test "npm: ubuntu pkg" {
    run vagga _run pkg-ubuntu resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/pkg-ubuntu)
    [[ $link = ".roots/pkg-ubuntu.daaf621e/root" ]]
}

@test "npm: precise pkg" {
    run vagga _run pkg-precise resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/pkg-precise)
    [[ $link = ".roots/pkg-precise.431bfbb9/root" ]]
}

@test "npm: alpine pkg" {
    run vagga _run pkg-alpine resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/pkg-alpine)
    [[ $link = ".roots/pkg-alpine.8512bd3c/root" ]]
}

@test "npm: default git" {
    run vagga _run git resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/git)
    [[ $link = ".roots/git.8b42a947/root" ]]
}

@test "npm: ubuntu git" {
    run vagga _run git-ubuntu resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/git-ubuntu)
    [[ $link = ".roots/git-ubuntu.035d58ac/root" ]]
}

@test "npm: alpine git" {
    run vagga _run git-alpine resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/git-alpine)
    [[ $link = ".roots/git-alpine.0afa9b2c/root" ]]
}

@test "npm: NpmDependencies" {
    run vagga _run npm-deps resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 124 ]]  # no resolve but has classnames --v
    [[ -f .vagga/npm-deps/usr/lib/node_modules/classnames/index.js ]]
    link=$(readlink .vagga/npm-deps)
    [[ $link = ".roots/npm-deps.a3931866/root" ]]
}
@test "npm: NpmDependencies dev" {
    run vagga _run npm-dev-deps resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/npm-dev-deps)
    [[ $link = ".roots/npm-dev-deps.55eea905/root" ]]
}

@test "npm: for alpine 3.6" {
    run vagga _run pkg-alpine-36 resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/pkg-alpine-36)
    [[ $link = ".roots/pkg-alpine-36.f51dedd3/root" ]]
}
