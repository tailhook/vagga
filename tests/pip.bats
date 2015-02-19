setup() {
    cd /work/tests/pip
}

@test "py2: ubuntu pkg" {
    run vagga _run py2-ubuntu urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/pkg-ubuntu)
    [[ $link = ".roots/py2-ubuntu.14bc28ff/root" ]]
}

@test "py2: alpine pkg" {
    run vagga _run py2-alpine urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/pkg-alpine)
    [[ $link = ".roots/pkg-alpine.864640c4/root" ]]
}

@test "py2: ubuntu git" {
    run vagga _run py2-git-ubuntu urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/pkg-ubuntu)
    [[ $link = ".roots/pkg-ubuntu.14bc28ff/root" ]]
}

@test "py2: alpine git" {
    run vagga _run py2-git-alpine urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/pkg-alpine)
    [[ $link = ".roots/pkg-alpine.864640c4/root" ]]
}

@test "py3: ubuntu pkg" {
    run vagga _run py3-ubuntu urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/pkg-ubuntu)
    [[ $link = ".roots/pkg-ubuntu.14bc28ff/root" ]]
}

@test "py3: ubuntu git" {
    run vagga _run py3-git-ubuntu urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/pkg-ubuntu)
    [[ $link = ".roots/pkg-ubuntu.14bc28ff/root" ]]
}
