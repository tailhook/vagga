setup() {
    cd /work/tests/badcmd
}

@test "badcmd: Bad command help message" {
    run vagga bad-cmd
    [[ $status -eq 126 ]]
    [[ ${lines[${#lines[@]}-2]} =~ .*'Validation Error: Field run is expected'.* ]]
    [[ ${lines[${#lines[@]}-1]} =~ .*'Decode error at .commands.bad-cmd: missing field `run`'.* ]]
}
