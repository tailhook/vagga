setup() {
    cd /work/tests/capsule
}

@test "capsule: Vagga in capsule runs" {
    run vagga vagga
    [[ $status = 127 ]]
    [[ $output = *"Recursive vagga"* ]]
}

@test "capsule: Vagga in capsule builds container" {
    run vagga vagga _capsule build v35-calc
    [[ $status = 0 ]]
    [[ $output = *"Container v35-calc (d6315e30) built"* ]]
}
