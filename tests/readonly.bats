# Readonly tests run with vagga outside of vagrant

@test "Vagga list works" {
    ./vagga _list
}

@test "Empty vagga has code 127" {
    run ./vagga
    [ "$status" -eq 127 ]
}

@test "Vagga build shell" {
    output=$(echo echo ok | ./vagga _build_shell)
    [ "$output" = "ok" ]
}
