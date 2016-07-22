setup() {
    cd /work/tests/inheritance
}

@test "inheritance: Deep container" {
    run vagga py
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/pythonic)
    [[ $link = ".roots/pythonic.a1389057/root" ]]
}

@test "inheritance: Run echo command" {
    run vagga echo hello
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
#    [[ $output = hello ]]
    [[ ${lines[${#lines[@]}-1]} = hello ]]
    link=$(readlink .vagga/base)
    [[ $link = ".roots/base.4ed6a479/root" ]]
}

@test "inheritance: Run bc" {
    run vagga calc 100*24
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/calc)
    [[ $link = ".roots/calc.02a0d1c1/root" ]]
}

@test "inheritance: Inherit from container with deep structure" {
    run vagga deep-cat
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "world" ]]
    link=$(readlink .vagga/sub)
    [[ $link = ".roots/sub.3f237d75/root" ]]
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
    [[ $link = ".roots/hellomount.e7eb747f/root" ]]
}

@test "inheritance: Build copy from mount" {
    run vagga hello-copy-from-mount
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello World!" ]]
    link=$(readlink .vagga/hellocopyfrommount)
    [[ $link = ".roots/hellocopyfrommount.f1485348/root" ]]
}

@test "inheritance: Build copy" {
    run vagga hello-copy
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello World!" ]]
    link=$(readlink .vagga/hellocopy)
    [[ $link = ".roots/hellocopy.b74c576d/root" ]]
}

@test "inheritance: Build copy file" {
    run vagga hello-copy-file
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello World!" ]]
    link=$(readlink .vagga/hellocopyfile)
    [[ $link = ".roots/hellocopyfile.b74c576d/root" ]]
}

@test "inheritance: Deep inheritance" {
    run vagga ok
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "10" ]]
    link=$(readlink .vagga/c10)
    [[ $link = ".roots/c10.4ed6a479/root" ]]
}
