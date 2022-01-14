setup() {
    cd /work/tests/ubuntu_release
}

@test "ubuntu-release: UbuntuRelease builds" {
    vagga _build ubuntu-release
    link=$(readlink .vagga/ubuntu-release)
    [[ $link = ".roots/ubuntu-release.a4152474/root" ]]
}

@test "ubuntu-release: echo command in ubuntu release" {
    run vagga echo-cmd hello
    [[ $status = 0 ]]
    [[ $output = hello ]]
    run vagga echo-cmd world
    [[ $status = 0 ]]
    [[ $output = world ]]
}

@test "ubuntu-release: echo shell in ubuntu release" {
    run vagga echo-shell
    [[ $status = 0 ]]
    [[ $output = "" ]]
    run vagga echo-shell hello
    [[ $status = 122 ]]
    [[ $output =~ "Unexpected argument" ]]
}

@test "ubuntu-release: echo shell with arguments in ubuntu release" {
    run vagga echo-shell-arg
    [[ $status = 0 ]]
    [[ $output = "" ]]
    run vagga echo-shell-arg hello
    [[ $status = 0 ]]
    [[ $output = "hello" ]]
}

@test "ubuntu-release: absent command in ubuntu release" {
    run vagga test something
    [[ $status -eq 121 ]]
    [[ $output =~ 'Command "test" not found and is not an alias' ]]
}

@test "ubuntu-release: bc in xenial by url" {
    run vagga xenial-calc 17*11
    link=$(readlink .vagga/xenial-url)
    echo "Container: $link"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "187" ]]
    [[ $link = ".roots/xenial-url.a3ad230f/root" ]]
}

@test "ubuntu-release: bc in ubuntu derived from release" {
    run vagga derived-calc 100*24
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/ubuntu-release-derive)
    [[ $link = ".roots/ubuntu-release-derive.ad41cc8a/root" ]]
}

@test "ubuntu-release: trusty bc in ubuntu release" {
    run vagga trusty-calc 23*7+3
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "164" ]]
    link=$(readlink .vagga/trusty-calc)
    [[ $link = ".roots/trusty-calc.22dbaca2/root" ]]
}

@test "ubuntu-release: VAGGAENV_* vars in ubuntu release" {
    VAGGAENV_TESTVAR=testvalue run vagga _run ubuntu-release printenv TESTVAR
    [[ $status -eq 0 ]]
    [[ $output = testvalue ]]
}

@test "ubuntu-release: set env in ubuntu release" {
    run vagga --environ TESTVAR=1value1 _run ubuntu-release printenv TESTVAR
    [[ $status -eq 0 ]]
    [[ $output = 1value1 ]]
}

@test "Test propagate env in ubuntu release" {
    TESTVAR=2value2 run vagga --use-env TESTVAR _run ubuntu-release printenv TESTVAR
    [[ $status -eq 0 ]]
    [[ $output = 2value2 ]]
}
