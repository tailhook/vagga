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

@test "capsule: Vagga in capsule runs command" {
    run vagga vagga _capsule run v35-calc bc < calc.txt
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
}
