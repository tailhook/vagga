setup() {
    cd /work/tests/ubuntu_release
}

@test "UbuntuRelease builds" {
    vagga _build utopic
    link=$(readlink .vagga/utopic)
    [[ $link = ".roots/utopic.a40d50e2/root" ]]
}

@test "Run echo command in ubuntu release" {
    run vagga echo-cmd hello
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = hello ]]
    run vagga echo-cmd world
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = world ]]
}

@test "Run echo shell in ubuntu release" {
    run vagga echo-shell
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = "" ]]
    run vagga echo-shell hello
    printf "%s\n" "${lines[@]}"
    [[ $status = 122 ]]
    [[ $output =~ "Unexpected argument" ]]
}

@test "Run echo shell with arguments in ubuntu release" {
    run vagga echo-shell-arg
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = "" ]]
    run vagga echo-shell-arg hello
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = "hello" ]]
}

@test "Run absent command in ubuntu release" {
    run vagga test something
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 121 ]]
    [[ $output =~ "Command test not found." ]]
}

@test "Run utopic bc in ubuntu release" {
    run vagga utopic-calc 100*24
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/utopic-calc)
    [[ $link = ".roots/utopic-calc.84c98d52/root" ]]
}

@test "Run trusty bc in ubuntu release" {
    run vagga trusty-calc 23*7+3
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "164" ]]
    link=$(readlink .vagga/trusty-calc)
    [[ $link = ".roots/trusty-calc.398044f3/root" ]]
}

@test "Test VAGGAENV_* vars in ubuntu release" {
    VAGGAENV_TESTVAR=testvalue run vagga _run utopic printenv TESTVAR
    [[ $status -eq 0 ]]
    [[ $output = testvalue ]]
}

@test "Test set env in ubuntu release" {
    run vagga --environ TESTVAR=1value1 _run utopic printenv TESTVAR
    [[ $status -eq 0 ]]
    [[ $output = 1value1 ]]
}

@test "Test propagate env in ubuntu release" {
    TESTVAR=2value2 run vagga --use-env TESTVAR _run utopic printenv TESTVAR
    [[ $status -eq 0 ]]
    [[ $output = 2value2 ]]
}
