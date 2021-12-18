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

@test "capsule: Vagga in capsule builds container and prints version" {
    run vagga vagga _capsule build --print-version v35-calc
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "v35-calc.d6315e30" ]]
}

@test "capsule: Vagga in capsule runs command" {
    run vagga vagga _capsule run v35-calc bc < calc.txt
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
}

@test "capsule: Vagga can run local capsule script" {
    run vagga vagga _capsule script ./script.sh 33+27
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "60" ]]
}

@test "capsule: Vagga can run remote capsule script" {
    scripturl="https://gist.githubusercontent.com/tailhook/0cf8adf45707c05702e5568b8d390ba9/raw/9f13527f7f94c27504b33f83e76da9514939b4c0/gistfile1.txt"
    run vagga vagga _capsule script "$scripturl" 33+27
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "60" ]]
}

@test "capsule: Vagga capsule download" {
    scripturl="https://gist.githubusercontent.com/tailhook/0cf8adf45707c05702e5568b8d390ba9/raw/9f13527f7f94c27504b33f83e76da9514939b4c0/gistfile1.txt"
    run vagga vagga _capsule download "$scripturl"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "/vagga/cache/downloads/698319cf-gistfile1.txt" ]]
}
