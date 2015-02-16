setup() {
    cd /work/tests/uidmap
}

@test "Too much uids" {
    run vagga _build too-much-uids
    [[ $status = 124 ]]
    [[ $output =~ "Number of allowed subuids is too small" ]]
}

@test "Too much gids" {
    run vagga _build too-much-gids
    [[ $status = 124 ]]
    [[ $output =~ "Number of allowed subgids is too small" ]]
}
