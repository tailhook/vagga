setup() {
    cd /work/tests/prerequisites
    vagga _build only
}

@test "version: One prerequisite" {
    run vagga two
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[0]} = "one" ]]
    [[ ${lines[1]} = "two" ]]
}

@test "version: Collapsing prerequisites" {
    run vagga four
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[0]} = "one" ]]
    [[ ${lines[1]} = "two" ]]
    [[ ${lines[2]} = "three" ]]
    [[ ${lines[3]} = "four" ]]
}

@test "version: Force order" {
    run vagga -m three two four
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[0]} = "one" ]]
    [[ ${lines[1]} = "three" ]]
    [[ ${lines[2]} = "two" ]]
    [[ ${lines[3]} = "four" ]]
}
