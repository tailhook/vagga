setup() {
    cd /work/tests/prerequisites
    vagga _build only
}

@test "prerequisites: One prerequisite" {
    run vagga two
    [[ $status = 0 ]]
    [[ ${lines[0]} = "one" ]]
    [[ ${lines[1]} = "two" ]]
}

@test "prerequisites: Collapsing prerequisites" {
    run vagga four
    [[ $status = 0 ]]
    [[ ${lines[0]} = "one" ]]
    [[ ${lines[1]} = "two" ]]
    [[ ${lines[2]} = "three" ]]
    [[ ${lines[3]} = "four" ]]
}

@test "prerequisites: Force order" {
    run vagga -m three two four
    [[ $status = 0 ]]
    [[ ${lines[0]} = "one" ]]
    [[ ${lines[1]} = "three" ]]
    [[ ${lines[2]} = "two" ]]
    [[ ${lines[3]} = "four" ]]
}

@test "prerequisites: Check persistent volume init" {
    run vagga persistent
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "world" ]]
}
