setup() {
    cd /work/tests/uidmap
}

@test "uidmap: Too much uids" {
    run vagga _build too-much-uids
    printf "%s\n" "${lines[@]}"
    echo "Status: $status"
    [[ $status = 121 ]]
    [[ $output =~ "Number of allowed subuids is too small" ]]
}

@test "uidmap: Too much gids" {
    run vagga _build too-much-gids
    printf "%s\n" "${lines[@]}"
    echo "Status: $status"
    [[ $status = 121 ]]
    [[ $output =~ "Number of allowed subgids is too small" ]]
}
