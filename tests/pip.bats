setup() {
    cd /work/tests/pip
}

@test "py2: ubuntu pkg" {
    run vagga _run py2-ubuntu urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py2-ubuntu)
    [[ $link = ".roots/py2-ubuntu.275bae9e/root" ]]
}

@test "py2: alpine pkg" {
    run vagga _run py2-alpine urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py2-alpine)
    [[ $link = ".roots/py2-alpine.202203b7/root" ]]
}

@test "py2: ubuntu git" {
    run vagga _run py2-git-ubuntu urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py2-git-ubuntu)
    [[ $link = ".roots/py2-git-ubuntu.05a2f3af/root" ]]
}

@test "py2: alpine git" {
    run vagga _run py2-git-alpine urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py2-git-alpine)
    [[ $link = ".roots/py2-git-alpine.05900679/root" ]]
}

@test "py3: ubuntu pkg" {
    run vagga _run py3-ubuntu urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py3-ubuntu)
    [[ $link = ".roots/py3-ubuntu.3bebbbd1/root" ]]
}

@test "py3: ubuntu py3.5" {
    vagga _build py35-ubuntu
    run vagga _run py35-ubuntu python3.5 -m urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py35-ubuntu)
    [[ $link = ".roots/py35-ubuntu.6bf2b457/root" ]]
}

@test "py3: ubuntu 15.04 py3.5" {
    vagga _build py35-ubuntu-15.04
    run vagga _run py35-ubuntu-15.04 python3.5 -m urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py35-ubuntu-15.04)
    [[ $link = ".roots/py35-ubuntu-15.04.08549ede/root" ]]
}

@test "py3: ubuntu git" {
    run vagga _run py3-git-ubuntu urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py3-git-ubuntu)
    [[ $link = ".roots/py3-git-ubuntu.1ffb6e90/root" ]]
}

@test "py2: ubuntu req.txt" {
    run vagga _run py2req-ubuntu urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py2req-ubuntu)
    [[ $link = ".roots/py2req-ubuntu.60e78f88/root" ]]
}

@test "py2: alpine req.txt" {
    run vagga _run py2req-alpine urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py2req-alpine)
    [[ $link = ".roots/py2req-alpine.2c5c0b81/root" ]]
}

@test "py3: ubuntu req-https.txt" {
    run vagga _build py3req-https-ubuntu
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/py3req-https-ubuntu)
    [[ $link = ".roots/py3req-https-ubuntu.b4474ef3/root" ]]
}

@test "py3: alpine req-https.txt" {
    run vagga _build py3req-https-alpine
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/py3req-https-alpine)
    [[ $link = ".roots/py3req-https-alpine.f05e1107/root" ]]
}

@test "py3: container inheritance" {
    run vagga _build py3req-inherit
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/py3req-inherit)
    [[ $link = ".roots/py3req-inherit.f05e1107/root" ]]
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

@test "py3: recursive requirements files" {
    run sh -c 'vagga _version_hash py3req-recursive-reqs -s'
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    result="${lines[@]}"
    echo 'injections' >> /work/tests/pip/include-short.txt
    run sh -c 'vagga _version_hash py3req-recursive-reqs -s'
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} != "$result" ]]
    run vagga _run py3req-recursive-reqs python3 -m urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
}
