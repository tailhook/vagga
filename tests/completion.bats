setup() {
    cd /work/tests/completion
}

@test "completion: user" {
    run vagga _compgen
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "dont_care no yes \
-E --env --environ -e --use-env --ignore-owner-check --no-build --no-version-check" ]]

    run vagga _compgen --
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "dont_care no yes \
-E --env --environ -e --use-env --ignore-owner-check --no-build --no-version-check" ]]

    run vagga _compgen -- -
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "-E --env --environ -e --use-env --ignore-owner-check --no-build --no-version-check" ]]

    run vagga _compgen -- --
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "--env --environ --use-env --ignore-owner-check --no-build --no-version-check" ]]

    run vagga _compgen -- d
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "dont_care" ]]

    run vagga _compgen -E test=123 --
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "dont_care no yes \
-E --env --environ -e --use-env --ignore-owner-check --no-build --no-version-check" ]]

    run vagga _compgen --no-build --
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "dont_care no yes \
-E --env --environ -e --use-env --ignore-owner-check --no-version-check" ]]
    
    run vagga _compgen -E test=123 -- d
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "dont_care" ]]

    run vagga _compgen -- does
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "" ]]

    run vagga _compgen -- --e
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "--env --environ" ]]
}

@test "completion: builtin" {
    run vagga _compgen -- _
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "_build _build_shell _clean _create_netns _destroy_netns \
_init_storage_dir _list _pack_image _run _run_in_netns _version_hash" ]]

    run vagga _compgen -- _r
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "_run _run_in_netns" ]]

    run vagga _compgen -- _ran
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "" ]]

    run vagga _compgen _build --
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "ubuntu --force" ]]

    run vagga _compgen _build -- -
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "--force" ]]

    run vagga _compgen _build -- --
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "--force" ]]

    run vagga _compgen _build --force --
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "ubuntu" ]]

    run vagga _compgen _run --
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "ubuntu -W --writable" ]]

    run vagga _compgen _run -- u
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "ubuntu" ]]

    run vagga _compgen --use-env HOME _run --writable -- u
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "ubuntu" ]]

    run vagga _compgen _run -- ud
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[@]} = "" ]]
}
