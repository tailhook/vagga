setup() {
    cd /work/tests/badcmd
}

@test "badcmd: Bad command help message" {
    run vagga bad-cmd
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 124 ]]
    [[ ${lines[${#lines[@]}-1]} =~ .*'Command has empty "run" parameter. Nothing to run.' ]]
}
