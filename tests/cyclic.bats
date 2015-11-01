setup() {
    cd /work/tests/cyclic
}

@test "cyclic: Crash prevention" {
    run vagga crash-me-not
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 126 ]]
    [[ ${lines[${#lines[@]}-1]} = 'Container "crash" has cyclic dependency' ]]
}
