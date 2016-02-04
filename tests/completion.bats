setup() {
    cd /work/tests/completion
}

@test "completion: no args" {
    run vagga _compgen
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "dont_care no yes" ]]
}

@test "completion: user" {
    run vagga _compgen --
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "dont_care no yes" ]]
}

@test "completion: user partial" {
    run vagga _compgen -- d
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "dont_care" ]]
}

@test "completion: user partial empty" {
    run vagga _compgen -- does
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "" ]]
}

@test "completion: builtin" {
    run vagga _compgen -- _
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "_build _build_shell _clean _create_netns _destroy_netns \
_init_storage_dir _list _pack_image _run _run_in_netns _version_hash" ]]
}

@test "completion: builtin partial" {
    run vagga _compgen -- _r
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "_run _run_in_netns" ]]
}

@test "completion: builtin partial empty" {
    run vagga _compgen -- _ran
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "" ]]
}

@test "completion: container" {
    run vagga _compgen _run --
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "ubuntu" ]]
}

@test "completion: container partial" {
    run vagga _compgen _run -- u
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "ubuntu" ]]
}

@test "completion: container partial empty" {
    run vagga _compgen _run -- ud
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "" ]]
}
