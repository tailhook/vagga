setup() {
    cd /work/tests/completion
}

@test "completion: global" {
    run vagga _compgen
    [[ $status = 0 ]]
    [[ ${lines[@]} = "dont_care no yes" ]]

    run vagga _compgen --
    [[ $status = 0 ]]
    [[ ${lines[@]} = "dont_care no yes" ]]

    run vagga _compgen -- -
    [[ $status = 0 ]]
    [[ ${lines[@]} = "-V --version -E --env --environ -e --use-env \
--ignore-owner-check --no-image-download --no-build \
--no-version-check --no-net --no-network --isolate-network" ]]

    run vagga _compgen -- --
    [[ $status = 0 ]]
    [[ ${lines[@]} = "--version --env --environ --use-env \
--ignore-owner-check --no-image-download --no-build \
--no-version-check --no-net --no-network --isolate-network" ]]

    run vagga _compgen -- y
    [[ $status = 0 ]]
    [[ ${lines[@]} = "yes" ]]

    run vagga _compgen yes --
    [[ $status = 0 ]]
    [[ ${lines[@]} = "" ]]

    run vagga _compgen -- d
    [[ $status = 0 ]]
    [[ ${lines[@]} = "dont_care" ]]

    run vagga _compgen --unknown-option --
    [[ $status = 0 ]]
    [[ ${lines[@]} = "dont_care no yes" ]]

    run vagga _compgen -E test=123 --
    [[ $status = 0 ]]
    [[ ${lines[@]} = "dont_care no yes" ]]

    run vagga _compgen --no-build -- -
    [[ $status = 0 ]]
    [[ ${lines[@]} = "-V --version -E --env --environ -e --use-env \
--ignore-owner-check --no-image-download --no-version-check \
--no-net --no-network --isolate-network" ]]

    run vagga _compgen -E test=123 -- -
    [[ $status = 0 ]]
    [[ ${lines[@]} = "-V --version -E --env --environ -e --use-env \
--ignore-owner-check --no-image-download --no-build \
--no-version-check --no-net --no-network --isolate-network" ]]

    run vagga _compgen -E test=123 -- d
    [[ $status = 0 ]]
    [[ ${lines[@]} = "dont_care" ]]

    run vagga _compgen -- does
    [[ $status = 0 ]]
    [[ ${lines[@]} = "" ]]

    run vagga _compgen -- --e
    [[ $status = 0 ]]
    [[ ${lines[@]} = "--env --environ" ]]
}

@test "completion: supervise" {
    run vagga _compgen dont_care --
    [[ $status = 0 ]]
    [[ ${lines[@]} = "--only --exclude --no-image-download \
--no-build --no-version-check" ]]

    run vagga _compgen --no-version-check -E HOME=/work dont_care --
    [[ $status = 0 ]]
    [[ ${lines[@]} = "--only --exclude --no-image-download \
--no-build --no-version-check" ]]

    run vagga _compgen dont_care -- --o
    [[ $status = 0 ]]
    [[ ${lines[@]} = "--only" ]]

    run vagga _compgen dont_care --no-build --
    [[ $status = 0 ]]
    [[ ${lines[@]} = "--only --exclude --no-image-download \
--no-version-check" ]]

    run vagga _compgen dont_care --only --
    [[ $status = 0 ]]
    [[ ${lines[@]} = "no yes" ]]

    run vagga _compgen dont_care --no-version-check --only --
    [[ $status = 0 ]]
    [[ ${lines[@]} = "no yes" ]]

    run vagga _compgen dont_care --only yes --
    [[ $status = 0 ]]
    [[ ${lines[@]} = "no" ]]

    run vagga _compgen dont_care --only yes -- -
    [[ $status = 0 ]]
    [[ ${lines[@]} = "--exclude --no-image-download --no-build \
--no-version-check" ]]

    run vagga _compgen dont_care --only yes --no-version-check -- -
    [[ $status = 0 ]]
    [[ ${lines[@]} = "--exclude --no-image-download --no-build" ]]
}

@test "completion: builtin" {
    run vagga _compgen -- _
    [[ $status = 0 ]]
    [[ ${lines[@]} = "_build _build_shell _clean _create_netns \
_destroy_netns _init_storage_dir _list _pack_image _push_image _run \
_run_in_netns _version_hash _check_overlayfs_support \
_base_dir _relative_work_dir _update_symlinks" ]]

    run vagga _compgen -- _r
    [[ $status = 0 ]]
    [[ ${lines[@]} = "_run _run_in_netns _relative_work_dir" ]]

    run vagga _compgen -- _ran
    [[ $status = 0 ]]
    [[ ${lines[@]} = "" ]]

    run vagga _compgen _build --
    [[ $status = 0 ]]
    [[ ${lines[@]} = "data ubuntu" ]]

    run vagga _compgen _build -- -
    [[ $status = 0 ]]
    [[ ${lines[@]} = "--force" ]]

    run vagga _compgen _build -- --
    [[ $status = 0 ]]
    [[ ${lines[@]} = "--force" ]]

    run vagga _compgen _build --force --
    [[ $status = 0 ]]
    [[ ${lines[@]} = "data ubuntu" ]]

    run vagga _compgen _run --
    [[ $status = 0 ]]
    [[ ${lines[@]} = "ubuntu" ]]

    run vagga _compgen _run -- -
    [[ $status = 0 ]]
    [[ ${lines[@]} = "-W --writable --no-image-download --no-build \
--no-version-check" ]]

    run vagga _compgen _run -- --
    [[ $status = 0 ]]
    [[ ${lines[@]} = "--writable --no-image-download --no-build \
--no-version-check" ]]

    run vagga _compgen _run -- u
    [[ $status = 0 ]]
    [[ ${lines[@]} = "ubuntu" ]]

    run vagga _compgen _run -W --
    [[ $status = 0 ]]
    [[ ${lines[@]} = "ubuntu" ]]

    run vagga _compgen --use-env HOME _run --writable -- u
    [[ $status = 0 ]]
    [[ ${lines[@]} = "ubuntu" ]]

    run vagga _compgen _run -- ud
    [[ $status = 0 ]]
    [[ ${lines[@]} = "" ]]

    run vagga _compgen _run unknown_container --
    [[ $status = 0 ]]
    [[ ${lines[@]} = "" ]]
}
