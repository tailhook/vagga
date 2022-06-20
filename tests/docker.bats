setup() {
    load '/bats/bats-support/load.bash'
    load '/bats/bats-assert/load.bash'
    cd /work/tests/docker
}

@test "docker: hello-world" {
    run vagga _build hello
    assert_success

    run vagga hello
    assert_success
    assert_line "Hello from Docker!"
}

@test "docker: python" {
    run vagga _build python
    assert_success

    run vagga zen
    assert_success
    assert_line "The Zen of Python, by Tim Peters"
}

@test "docker: java" {
    run vagga _build java
    assert_success

    run vagga _run java /usr/local/openjdk-17/bin/java -version
    assert_success
    assert_line -p "17.0.1"
}

@test "docker: registry" {
    run vagga _build buildah
    assert_success
    run vagga _build registry
    assert_success
    run vagga _build test-image
    assert_equal "$status" 121

    run vagga buildah --version
    assert_success

    run vagga build-test-image
    assert_success

    run vagga --isolate-network push-and-build-test-image
    assert_success
    link=$(readlink .vagga/test-image)
    assert_equal "$link" ".roots/test-image.f62f1455/root"
    [[ ! -e .vagga/test-image/will-be-deleted ]]
    [[ -f .vagga/test-image/test/vagga.yaml ]]
    [[ ! -e .vagga/test-image/test/Dockerfile ]]
    assert_equal "$(cat .vagga/test-image/hello.txt)" "Hello world!"
    [[ -L .vagga/test-image/hi.txt ]]
    assert_equal "$(cat .vagga/test-image/hi.txt)" "Hello world!"
    assert_equal "$(cat .vagga/test-image/see-you.txt)" "Bye-bye!"
    assert_equal "$(stat -c '%i' .vagga/test-image/see-you.txt)" "$(stat -c '%i' .vagga/test-image/bye-bye.txt)"
    [[ -f .vagga/test-image/empty ]]
}
