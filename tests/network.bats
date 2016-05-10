setup() {
    cd /work/tests/network
    vagga _build py
}

@test "network: bind port" {
    vagga bind &
    sleep 0.05
    run vagga connect
    kill %1
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[0]} = "hello world!" ]]
}

@test "network: bind port in supervise" {
    vagga superbind &
    sleep 0.05
    run vagga connect
    kill %1
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[0]} = "hello world!" ]]
}
