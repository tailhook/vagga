setup() {
    cd /work/tests/subconfig
}

@test "subconfig: Run bc" {
    run vagga calc 100*24
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/subdir)
    [[ $link = ".roots/subdir.83b9845a/root" ]]
}

@test "subconfig: docker-raw" {
    run vagga _run docker-raw urp -Q key=val http://example.com
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/docker-raw)
    [[ $link = ".roots/docker-raw.7d5f876a/root" ]]
}

@test "subconfig: docker-smart" {
    run vagga _run docker-smart urp -Q key=val http://example.com
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/docker-smart)
    [[ $link = ".roots/docker-smart.fb95b4a3/root" ]]
}
