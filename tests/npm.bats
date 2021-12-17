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

@test "npm: bionic pkg" {
    run vagga _run pkg-bionic resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/pkg-bionic)
    printf "Link:  %s" "$link"
    [[ $link = ".roots/pkg-bionic.75d76405/root" ]]
}

@test "npm: xenial pkg" {
    run vagga _run pkg-xenial resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/pkg-xenial)
    printf "Link:  %s" "$link"
    [[ $link = ".roots/pkg-xenial.f8666b62/root" ]]
}

@test "npm: alpine pkg" {
    run vagga _run pkg-alpine resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/pkg-alpine)
    [[ $link = ".roots/pkg-alpine.825ac0ab/root" ]]
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
    [[ $link = ".roots/git-alpine.0445ebc1/root" ]]
}

@test "npm: NpmDependencies" {
    run vagga _run npm-deps resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 124 ]]  # no resolve but has classnames --v
    [[ -f .vagga/npm-deps/usr/lib/node_modules/classnames/index.js ]]
    link=$(readlink .vagga/npm-deps)
    [[ $link = ".roots/npm-deps.f9fadad7/root" ]]
}
@test "npm: NpmDependencies dev" {
    run vagga _run npm-dev-deps resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/npm-dev-deps)
    [[ $link = ".roots/npm-dev-deps.f4c17a3c/root" ]]
}

@test "npm: unsupported alpine version" {
    run vagga _run pkg-alpine-36 resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 121 ]]
    [[ $output = *"Alpine v3.6 does not support npm"* ]]
}
