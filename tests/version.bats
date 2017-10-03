setup() {
    cd /work/tests/version
}

@test "version: Check minimum version" {
    run vagga run something
    printf "%s\n" "${lines[@]}"
    [[ $status = 126 ]]
    [[ ${lines[0]} = *'Minimum Vagga Error: Please upgrade vagga to at least "9999.0.0"' ]]
    [[ ${lines[1]} = *'Validation Error: The tag UnknownCommand is not expected' ]]
}

@test "version: Show version" {
    run vagga --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = 'v'* ]]

    cd /
    run vagga --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = 'v'* ]]
}
