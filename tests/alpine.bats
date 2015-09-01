setup() {
    cd /work/tests/alpine
}

@test "alpine: Alpine builds" {
    vagga _build v31
    link=$(readlink .vagga/v31)
    [[ $link = ".roots/v31.f87ff413/root" ]]
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

@test "alpine: Run bc on v3.2" {
    run vagga v32-calc 100*24
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/v32-calc)
    [[ $link = ".roots/v32-calc.39b2646e/root" ]]
}

@test "alpine: Run bc on v3.1" {
    run vagga v31-calc 100*24
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/v31-calc)
    [[ $link = ".roots/v31-calc.dcc4a56e/root" ]]
}

@test "alpine: Run bc on v3.0" {
    run vagga v30-calc 23*7+3
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "164" ]]
    link=$(readlink .vagga/v30-calc)
    [[ $link = ".roots/v30-calc.45353994/root" ]]
}
