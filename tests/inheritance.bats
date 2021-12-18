setup() {
    cd /work/tests/inheritance
}

@test "inheritance: Deep container" {
    run vagga py
    [[ $status = 0 ]]
    link=$(readlink .vagga/pythonic)
    [[ $link = ".roots/pythonic.db184093/root" ]]
}

@test "inheritance: Run echo command" {
    run vagga echo hello
    [[ $status = 0 ]]
#    [[ $output = hello ]]
    [[ ${lines[${#lines[@]}-1]} = hello ]]
    link=$(readlink .vagga/base)
    [[ $link = ".roots/base.d304a005/root" ]]
}

@test "inheritance: Run bc" {
    run vagga calc 100*24
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/calc)
    [[ $link = ".roots/calc.5af5e7a3/root" ]]
}

@test "inheritance: Inherit from container with deep structure" {
    run vagga deep-cat
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "world" ]]
    link=$(readlink .vagga/sub)
    [[ $link = ".roots/sub.9f9d0b57/root" ]]
}

@test "inheritance: Test hardlink copy of the deep structure" {
    run vagga deep-cat-copy
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "world" ]]
}

@test "inheritance: Build mount" {
    run vagga hello-mount
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello World!" ]]
    link=$(readlink .vagga/hellomount)
    [[ $link = ".roots/hellomount.9c7c2e59/root" ]]
}

@test "inheritance: Build copy from mount" {
    run vagga hello-copy-from-mount
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello World!" ]]
    link=$(readlink .vagga/hellocopyfrommount)
    [[ $link = ".roots/hellocopyfrommount.70fc36a1/root" ]]
}

@test "inheritance: Build copy" {
    run vagga hello-copy
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello World!" ]]
    link=$(readlink .vagga/hellocopy)
    [[ $link = ".roots/hellocopy.0ce5ff73/root" ]]
}

@test "inheritance: Build copy contenthash" {
    run vagga hello-copy-ch
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello World!" ]]
    link=$(readlink .vagga/hellocopy-contenthash)
    [[ $link = ".roots/hellocopy-contenthash.96121dc4/root" ]]
}

@test "inheritance: Build copy rules" {
    run vagga hello-copy-rules
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello World!" ]]
    link=$(readlink .vagga/hellocopy-rules)
    [[ $link = ".roots/hellocopy-rules.1c88be7f/root" ]]
}

@test "inheritance: Build copy file" {
    run vagga hello-copy-file
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello World!" ]]
    link=$(readlink .vagga/hellocopyfile)
    [[ $link = ".roots/hellocopyfile.0ce5ff73/root" ]]
}

@test "inheritance: Build copy file with contenthash" {
    run vagga hello-copy-file-ch
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello World!" ]]
    link=$(readlink .vagga/hellocopyfile-contenthash)
    [[ $link = ".roots/hellocopyfile-contenthash.96121dc4/root" ]]
}

@test "inheritance: Deep inheritance" {
    run vagga ok
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "10" ]]
    link=$(readlink .vagga/c10)
    [[ $link = ".roots/c10.e264500e/root" ]]
}
