setup() {
    cd /work/tests/inheritance
}

@test "inheritance: Deep container" {
    run vagga py
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/pythonic)
    [[ $link = ".roots/pythonic.659804b8/root" ]]
}

@test "inheritance: Run echo command" {
    run vagga echo hello
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
#    [[ $output = hello ]]
    [[ ${lines[${#lines[@]}-1]} = hello ]]
}

@test "inheritance: Run bc" {
    run vagga calc 100*24
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/calc)
    [[ $link = ".roots/calc.dcc4a56e/root" ]]
}

@test "inheritance: Inherit from container with deep structure" {
    run vagga deep-cat
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "world" ]]
    link=$(readlink .vagga/sub)
    [[ $link = ".roots/sub.e8bf6100/root" ]]
}

@test "inheritance: Test hardlink copy of the deep structure" {
    run vagga deep-cat-copy
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "world" ]]
}

@test "inheritance: Build mount" {
    run vagga hello-mount
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello World!" ]]
    link=$(readlink .vagga/hellomount)
    [[ $link = ".roots/hellomount.93d4bd97/root" ]]
}

@test "inheritance: Build copy" {
    run vagga hello-copy
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello World!" ]]
    link=$(readlink .vagga/hellocopy)
    [[ $link = ".roots/hellocopy.208979ad/root" ]]
}
