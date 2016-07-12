setup() {
    cd /work/tests/npm
}

@test "npm: default pkg" {
    run vagga _run pkg resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/pkg)
    [[ $link = ".roots/pkg.1f967d43/root" ]]
}

@test "npm: ubuntu pkg" {
    run vagga _run pkg-ubuntu resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/pkg-ubuntu)
    [[ $link = ".roots/pkg-ubuntu.c619f50b/root" ]]
}

@test "npm: precise pkg" {
    run vagga _run pkg-precise resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/pkg-precise)
    [[ $link = ".roots/pkg-precise.ebadbadc/root" ]]
}

@test "npm: alpine pkg" {
    run vagga _run pkg-alpine resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/pkg-alpine)
    [[ $link = ".roots/pkg-alpine.d2132f1a/root" ]]
}

@test "npm: default git" {
    run vagga _run git resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/git)
    [[ $link = ".roots/git.ddbc7338/root" ]]
}

@test "npm: ubuntu git" {
    run vagga _run git-ubuntu resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/git-ubuntu)
    [[ $link = ".roots/git-ubuntu.4a0493c5/root" ]]
}

@test "npm: alpine git" {
    run vagga _run git-alpine resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/git-alpine)
    [[ $link = ".roots/git-alpine.43611672/root" ]]
}

@test "npm: NpmDependencies" {
    run vagga _run npm-deps resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 124 ]]  # no resolve but has classnames --v
    [[ -f .vagga/npm-deps/usr/lib/node_modules/classnames/index.js ]]
    link=$(readlink .vagga/npm-deps)
    [[ $link = ".roots/npm-deps.ae17e04b/root" ]]
}
@test "npm: NpmDependencies dev" {
    run vagga _run npm-dev-deps resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/npm-dev-deps)
    [[ $link = ".roots/npm-dev-deps.ebd5e264/root" ]]
}
