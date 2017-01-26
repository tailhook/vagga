setup() {
    cd /work/tests/capsule
}

@test "capsule: Vagga in capsule runs" {
    run vagga vagga
    [[ $status = 127 ]]
    [[ $output = *"Recursive vagga"* ]]
}
