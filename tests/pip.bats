setup() {
    cd /work/tests/pip
}

@test "py2: ubuntu pkg" {
    run vagga _run py2-ubuntu urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py2-ubuntu)
    [[ $link = ".roots/py2-ubuntu.f97edeec/root" ]]
}

@test "py2: alpine pkg" {
    run vagga _run py2-alpine urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py2-alpine)
    [[ $link = ".roots/py2-alpine.1e5e9fd7/root" ]]
}

@test "py2: ubuntu git" {
    run vagga _run py2-git-ubuntu urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py2-git-ubuntu)
    [[ $link = ".roots/py2-git-ubuntu.3bfa83a6/root" ]]
}

@test "py2: alpine git" {
    run vagga _run py2-git-alpine urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py2-git-alpine)
    [[ $link = ".roots/py2-git-alpine.4839be1d/root" ]]
}

@test "py3: ubuntu pkg" {
    run vagga _run py3-ubuntu urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py3-ubuntu)
    [[ $link = ".roots/py3-ubuntu.db09bc7e/root" ]]
}

@test "py3: ubuntu py3.5" {
    vagga _build py35-ubuntu
    run vagga _run py35-ubuntu python3.5 -m urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py35-ubuntu)
    [[ $link = ".roots/py35-ubuntu.5d76cc91/root" ]]
}

@test "py3: ubuntu git" {
    run vagga _run py3-git-ubuntu urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py3-git-ubuntu)
    [[ $link = ".roots/py3-git-ubuntu.53dd180a/root" ]]
}

@test "py2: ubuntu req.txt" {
    run vagga _run py2req-ubuntu urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py2req-ubuntu)
    [[ $link = ".roots/py2req-ubuntu.abeddcee/root" ]]
}

@test "py2: alpine req.txt" {
    run vagga _run py2req-alpine urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/py2req-alpine)
    [[ $link = ".roots/py2req-alpine.9c3e51f3/root" ]]
}

@test "py3: ubuntu req-https.txt" {
    run vagga _build py3req-https-ubuntu
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/py3req-https-ubuntu)
    [[ $link = ".roots/py3req-https-ubuntu.39fc6dcd/root" ]]
}

@test "py3: alpine req-https.txt" {
    run vagga _build py3req-https-alpine
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/py3req-https-alpine)
    [[ $link = ".roots/py3req-https-alpine.fb29b883/root" ]]
}

@test "py3: container inheritance" {
    run vagga _build py3req-inherit
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/py3req-inherit)
    [[ $link = ".roots/py3req-inherit.51b2cf8f/root" ]]
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
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = $'pty_copy\r' ]]
}

@test "py3: recursive requirements files" {
    echo '-r requirements.txt' > /work/tests/pip/include-short.txt
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

@test "py3: pip dependencies" {
    run vagga _build pip-deps
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/pip-deps)
    [[ $link = ".roots/pip-deps.ff73d42f/root" ]]
}
