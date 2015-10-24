setup() {
    cd /work/tests/inheritance
}

@test "inheritance: Deep container" {
    run vagga py
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/pythonic)
    [[ $link = ".roots/pythonic.659804b8/root" ]]
}

@test "inheritance: Run echo command" {
    run vagga echo hello
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
#    [[ $output = hello ]]
    [[ ${lines[${#lines[@]}-1]} = hello ]]
}

@test "inheritance: Run bc" {
    run vagga calc 100*24
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/calc)
    [[ $link = ".roots/calc.dcc4a56e/root" ]]
}

@test "inheritance: Inherit from container with deep structure" {
    run vagga _build sub
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-2]} = "world" ]]
    link=$(readlink .vagga/sub)
    [[ $link = ".roots/sub.88e9d314/root" ]]
}
