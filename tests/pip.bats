setup() {
    cd /work/tests/pip
}

@test "py2: ubuntu pkg" {
    run vagga _run py2-ubuntu urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py2-ubuntu)
    [[ $link = ".roots/py2-ubuntu.b6bc38d1/root" ]]
}

@test "py2: alpine pkg" {
    run vagga _run py2-alpine urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py2-alpine)
    [[ $link = ".roots/py2-alpine.a7327653/root" ]]
}

@test "py2: ubuntu git" {
    run vagga _run py2-git-ubuntu urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py2-git-ubuntu)
    [[ $link = ".roots/py2-git-ubuntu.aedb2403/root" ]]
}

@test "py2: alpine git" {
    run vagga _run py2-git-alpine urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py2-git-alpine)
    [[ $link = ".roots/py2-git-alpine.569f9a5e/root" ]]
}

@test "py3: ubuntu pkg" {
    run vagga _run py3-ubuntu urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py3-ubuntu)
    [[ $link = ".roots/py3-ubuntu.c2f5a64e/root" ]]
}

@test "py3: ubuntu py3.5" {
    vagga _build py35-ubuntu
    run vagga _run py35-ubuntu python3.5 -m urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py35-ubuntu)
    [[ $link = ".roots/py35-ubuntu.384faa53/root" ]]
}

@test "py3: ubuntu 15.04 py3.5" {
    vagga _build py35-ubuntu-15.04
    run vagga _run py35-ubuntu-15.04 python3.5 -m urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py35-ubuntu-15.04)
    [[ $link = ".roots/py35-ubuntu-15.04.e00f419a/root" ]]
}

@test "py3: ubuntu git" {
    run vagga _run py3-git-ubuntu urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py3-git-ubuntu)
    [[ $link = ".roots/py3-git-ubuntu.453926f2/root" ]]
}

@test "py2: ubuntu req.txt" {
    run vagga _run py2req-ubuntu urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py2req-ubuntu)
    [[ $link = ".roots/py2req-ubuntu.1730f1da/root" ]]
}

@test "py2: alpine req.txt" {
    run vagga _run py2req-alpine urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py2req-alpine)
    [[ $link = ".roots/py2req-alpine.eb8c5b79/root" ]]
}

@test "py3: ubuntu req-https.txt" {
    run vagga _build py3req-https-ubuntu
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/py3req-https-ubuntu)
    [[ $link = ".roots/py3req-https-ubuntu.086b02bf/root" ]]
}

@test "py3: alpine req-https.txt" {
    run vagga _build py3req-https-alpine
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/py3req-https-alpine)
    [[ $link = ".roots/py3req-https-alpine.356eb50e/root" ]]
}

@test "py3: container inheritance" {
    run vagga _build py3req-inherit
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/py3req-inherit)
    [[ $link = ".roots/py3req-inherit.356eb50e/root" ]]
}

@test "pip: C dependencies caching" {
    vagga _build ubuntu-lxml
    vagga _build alpine-lxml
    run vagga _run alpine-lxml python3 -c "from lxml import etree"
    printf "%s\n" "${lines[@]}"
    echo STATUS "$status"
    [[ $status = 0 ]]
}

@test "py3: pty works" {
    run vagga pty-output
    printf "%s\n" "${lines[@]}"
    echo STATUS "$status"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = $'pty_copy\r' ]]
}
