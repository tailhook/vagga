setup() {
    cd /work/tests/alpine
}

@test "alpine: Alpine builds" {
    vagga _build v31
    link=$(readlink .vagga/v31)
    [[ $link = ".roots/v31.3bbd8dfc/root" ]]
}

@test "alpine: Check stdout" {
    run echo $(vagga v33-tar -cz vagga.yaml | tar -zt)
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/v33-tar)
    [[ $link = ".roots/v33-tar.308cf7fd/root" ]]
    [[ $output = "vagga.yaml" ]]
}

@test "alpine: Check version" {
    run vagga _build alpine-check-version
    printf "%s\n" "${lines[@]}"
    [[ $status = 121 ]]
    [[ $output = *"Error checking alpine version"* ]]
}

@test "alpine: Alpine repo" {
    run vagga _build alpine-repo
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/alpine-repo)
    [[ $link = ".roots/alpine-repo.a3bfac74/root" ]]
    run vagga _run alpine-repo tini -h
    [[ $output = *"tini (version 0.9.0)"* ]]
}

@test "alpine: Run echo command" {
    run vagga echo-cmd hello
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = hello ]]
    run vagga echo-cmd world
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = world ]]
}

@test "alpine: Run bc on v3.4" {
    run vagga v34-calc 100*24
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/v34-calc)
    [[ $link = ".roots/v34-calc.02a0d1c1/root" ]]
}

@test "alpine: Run bc on v3.3" {
    run vagga v33-calc 100*24
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/v33-calc)
    [[ $link = ".roots/v33-calc.52ba709f/root" ]]
}

@test "alpine: Run bc on v3.2" {
    run vagga v32-calc 100*24
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/v32-calc)
    [[ $link = ".roots/v32-calc.a3ffc64f/root" ]]
}

@test "alpine: Run bc on v3.1" {
    run vagga v31-calc 100*24
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/v31-calc)
    [[ $link = ".roots/v31-calc.c42da512/root" ]]
}

@test "alpine: Run bc on v3.0" {
    run vagga v30-calc 23*7+3
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "164" ]]
    link=$(readlink .vagga/v30-calc)
    [[ $link = ".roots/v30-calc.a60099cd/root" ]]
}

@test "alpine: Run vagga inside alpine" {
    cp ../../vagga vagga_inside_alpine/
    cp ../../apk vagga_inside_alpine/
    cp ../../busybox vagga_inside_alpine/
    cp ../../alpine-keys.apk vagga_inside_alpine/

    run vagga vagga-alpine
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-2]} = 6ea38cf8 ]]
    [[ ${lines[${#lines[@]}-1]} = 6ea38cf8bd751ac737a41c6e1ddb4b87a804f8e562c30064ec42941005b7bc6f ]]
}
