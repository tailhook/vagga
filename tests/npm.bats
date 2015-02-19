setup() {
    cd /work/tests/npm
}

@test "npm: default pkg" {
    run vagga _run pkg resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/pkg)
    [[ $link = ".roots/pkg.59688055/root" ]]
}

@test "npm: ubuntu pkg" {
    run vagga _run pkg-ubuntu resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/pkg-ubuntu)
    [[ $link = ".roots/pkg-ubuntu.14bc28ff/root" ]]
}

@test "npm: alpine pkg" {
    run vagga _run pkg-alpine resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/pkg-alpine)
    [[ $link = ".roots/pkg-alpine.864640c4/root" ]]
}

@test "npm: default git" {
    run vagga _run git resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/git)
    [[ $link = ".roots/git.0d63daec/root" ]]
}

@test "npm: ubuntu git" {
    run vagga _run git-ubuntu resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/git-ubuntu)
    [[ $link = ".roots/git-ubuntu.a3bf710f/root" ]]
}

@test "npm: alpine git" {
    run vagga _run git-alpine resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/git-alpine)
    [[ $link = ".roots/git-alpine.d6b3d182/root" ]]
}
