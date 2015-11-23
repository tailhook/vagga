setup() {
    cd /work/tests/version
}

@test "version: Check minimum version" {
    run vagga run something
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 126 ]]
    [[ ${lines[${#lines[@]}-1]} = 'Please upgrade vagga to at least "9999.0.0"' ]]
}
