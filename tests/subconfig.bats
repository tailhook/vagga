setup() {
    cd /work/tests/subconfig
}

@test "subconfig: Run bc" {
    run vagga calc 100*24
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/subdir)
    [[ $link = ".roots/subdir.68a08ebe/root" ]]
}

@test "subconfig: docker-raw" {
    run vagga _run docker-raw urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/docker-raw)
    [[ $link = ".roots/docker-raw.91190e88/root" ]]
}

@test "subconfig: docker-smart" {
    run vagga _run docker-smart urp -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/docker-smart)
    [[ $link = ".roots/docker-smart.e2fa942b/root" ]]
}
