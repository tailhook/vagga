setup() {
    cd /work/tests/ubuntu
}

@test "Ubuntu builds" {
    vagga _build trusty
    link=$(readlink .vagga/trusty)
    [[ $link = ".roots/trusty.52f1f091/root" ]]
}

@test "Run echo command" {
    run vagga echo-cmd hello
    [[ $status = 0 ]]
    [[ $output = hello ]]
    run vagga echo-cmd world
    [[ $status = 0 ]]
    [[ $output = world ]]
}

@test "Run echo shell" {
    run vagga echo-shell
    [[ $status = 0 ]]
    [[ $output = "" ]]
    run vagga echo-shell hello
    [[ $status = 122 ]]
    [[ $output =~ "Unexpected argument" ]]
}

@test "Run echo shell with arguments" {
    run vagga echo-shell-arg
    [[ $status = 0 ]]
    [[ $output = "" ]]
    run vagga echo-shell-arg hello
    echo "OUTPUT (($output))"
    [[ $status = 0 ]]
    [[ $output = "hello" ]]
}

@test "Run absent command" {
    run vagga test something
    [[ $status -eq 121 ]]
    [[ $output =~ "Command test not found." ]]
}

@test "Run trusty bc" {
    run vagga trusty-calc 100*24
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/trusty-calc)
    [[ $link = ".roots/trusty-calc.97c8fccd/root" ]]
}

@test "Run precise bc" {
    run vagga precise-calc 23*7+3
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "164" ]]
    link=$(readlink .vagga/precise-calc)
    [[ $link = ".roots/precise-calc.ba9759ba/root" ]]
}
