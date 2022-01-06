setup() {
    load '/bats/bats-support/load.bash'
    load '/bats/bats-assert/load.bash'
    cd /work/tests/ubuntu
}

@test "ubuntu: builds" {
    run env RUST_LOG=info vagga _build --force trusty
    assert_success
    link=$(readlink .vagga/trusty)
    assert_equal $link ".roots/trusty.010798c2/root"
    assert_line -p "Eatmydata activated"
}

@test "ubuntu: i386 builds" {
    run env RUST_LOG=info vagga _build --force xenial-i386
    assert_success
    link=$(readlink .vagga/xenial-i386)
    assert_equal $link ".roots/xenial-i386.30fb2ea2/root"
    assert_line -p "Eatmydata activated"
}

@test "ubuntu: apt cache" {
    fortune_pkgs="/work/tmp/cache/apt-cache/archives/fortune-mod_*.deb"
    rm -f $fortune_pkgs
    [[ $(ls -l $fortune_pkgs | wc -l) = "0" ]]

    run vagga _build apt-cache
    [[ $status = 0 ]]
    [[ $(readlink .vagga/apt-cache) = ".roots/apt-cache.835afa14/root" ]]
    [[ $(ls -l $fortune_pkgs | wc -l) = "1" ]]
}

@test "ubuntu: run echo command" {
    run vagga echo-cmd hello
    [[ $status = 0 ]]
    [[ $output = hello ]]
    run vagga echo-cmd world
    [[ $status = 0 ]]
    [[ $output = world ]]
}

@test "ubuntu: run echo shell" {
    run vagga echo-shell
    [[ $status = 0 ]]
    [[ $output = "" ]]
    run vagga echo-shell hello
    [[ $status = 122 ]]
    [[ $output =~ "Unexpected argument" ]]
}

@test "ubuntu: run echo shell with arguments" {
    run vagga echo-shell-arg
    [[ $status = 0 ]]
    [[ $output = "" ]]
    run vagga echo-shell-arg hello
    [[ $status = 0 ]]
    [[ $output = "hello" ]]
}

@test "ubuntu: run absent command" {
    run vagga test something
    [[ $status -eq 121 ]]
    [[ $output =~ 'Command "test" not found and is not an alias' ]]
}

@test "ubuntu: check arch support" {
    run vagga check-arch
    [[ $status = 0 ]]
    [[ $output = i386 ]]
}

@test "ubuntu: run trusty bc" {
    run vagga trusty-calc 100*24
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/trusty)
    [[ $link = ".roots/trusty.010798c2/root" ]]
}

@test "ubuntu: run xenial bc" {
    run vagga xenial-calc 23*7+3
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "164" ]]
    link=$(readlink .vagga/xenial-calc)
    [[ $link = ".roots/xenial-calc.321f6a11/root" ]]
}

@test "ubuntu: BuildDeps with version" {
    run vagga _build build-deps-with-version
    [[ $status = 0 ]]
    [[ $output = *"480191"* ]]
    link=$(readlink .vagga/build-deps-with-version)
    [[ $link = ".roots/build-deps-with-version.293fcc59/root" ]]

    run vagga _run build-deps-with-version bc
    [[ $status = 124 ]]
}

@test "ubuntu: focal universe" {
    run env RUST_LOG=info vagga _build --force ubuntu-universe
    assert_success
    link=$(readlink .vagga/ubuntu-universe)
    assert_equal $link ".roots/ubuntu-universe.cf089a9f/root"
    assert_line -p "Eatmydata activated"

    run vagga _run ubuntu-universe /usr/games/cowsay "Have you mooed today?"
    [[ $status = 0 ]]
    [[ $output = *"Have you mooed today?"* ]]
}

@test "ubuntu: VAGGAENV_* vars" {
    VAGGAENV_TESTVAR=testvalue run vagga _run trusty printenv TESTVAR
    [[ $status -eq 0 ]]
    [[ $output = testvalue ]]
}

@test "ubuntu: set env" {
    run vagga --environ TESTVAR=1value1 _run trusty printenv TESTVAR
    [[ $status -eq 0 ]]
    [[ $output = 1value1 ]]
}

@test "ubuntu: propagate env" {
    TESTVAR=2value2 run vagga --use-env TESTVAR _run trusty printenv TESTVAR
    [[ $status -eq 0 ]]
    [[ $output = 2value2 ]]
}

@test "ubuntu: the chfn just works (i.e. a no-op)" {
    run vagga rename-me
    [[ $status -eq 0 ]]
    [[ $output = "" ]]
}

@test "ubuntu: builddeps needed for other packages" {
    run vagga checkinstall -v
    [[ $status -eq 0 ]]
    [[ $output != "" ]]
    link=$(readlink .vagga/dependency-conflict)
    [[ $link = ".roots/dependency-conflict.2dea7ae3/root" ]]
}

@test "ubuntu: install from ppa" {
    run vagga _run ppa redis-cli --version
    [[ $status -eq 0 ]]
    [[ $output != "" ]]
    link=$(readlink .vagga/ppa)
    [[ $link = ".roots/ppa.ff590a0f/root" ]]
}

@test "ubuntu: install from ppa in bionic" {
    run vagga _run ppa_bionic redis-cli --version
    [[ $status -eq 0 ]]
    [[ $output != "" ]]
    link=$(readlink .vagga/ppa_bionic)
    [[ $link = ".roots/ppa_bionic.85fbff22/root" ]]
}

@test "ubuntu: UbuntuRepo minimal" {
    run vagga _build ubuntu-repo-minimal
    link=$(readlink .vagga/ubuntu-repo-minimal)
    [[ $link = ".roots/ubuntu-repo-minimal.4c867b7a/root" ]]

    repo_line=$(cat ".vagga/ubuntu-repo-minimal/etc/apt/sources.list.d/8afb3430-xenial.list")
    [[ $repo_line = *" xenial universe" ]]

    run vagga _run ubuntu-repo-minimal /usr/games/cowsay "Have you mooed today?"
    [[ $status = 0 ]]
    [[ $output = *"Have you mooed today?"* ]]
}

@test "ubuntu: UbuntuRepo full" {
    run vagga _build ubuntu-repo-full
    link=$(readlink .vagga/ubuntu-repo-full)
    [[ $link = ".roots/ubuntu-repo-full.71fe190e/root" ]]

    repo_line=$(cat ".vagga/ubuntu-repo-full/etc/apt/sources.list.d/2efc24ff-vagga.list")
    [[ $repo_line = "deb [trusted=yes] http://ubuntu.zerogw.com vagga main" ]]

    run vagga _run ubuntu-repo-full vagga --version
    [[ $status = 0 ]]
    [[ $output = "v0.8.1" ]]
}

@test "ubuntu: UbuntuRepo https" {
    run vagga _build ubuntu-repo-https-sub
    link=$(readlink .vagga/ubuntu-repo-https-sub)
    [[ $link = ".roots/ubuntu-repo-https-sub.e23819c1/root" ]]

    repo_line=$(cat ".vagga/ubuntu-repo-https-sub/etc/apt/sources.list.d/94acf98e-xenial.list")
    [[ $repo_line = "deb https://deb.nodesource.com/node_5.x xenial main" ]]

    run vagga _run ubuntu-repo-https-sub node --version
    [[ $status = 0 ]]
    [[ $output = "v5."* ]]
}

@test "ubuntu: Repo simple" {
    run vagga _build repo-simple
    link=$(readlink .vagga/repo-simple)
    [[ $link = ".roots/repo-simple.382949ab/root" ]]

    repo_line=$(cat ".vagga/repo-simple/etc/apt/sources.list.d/8afb3430-xenial.list")
    [[ $repo_line = *" xenial universe" ]]

    run vagga _run repo-simple banner Wonderful
    [[ $output = "#     #"* ]]
}

@test "ubuntu: Repo with suite" {
    run vagga _build repo-with-suite
    link=$(readlink .vagga/repo-with-suite)
    [[ $link = ".roots/repo-with-suite.31742766/root" ]]

    repo_line=$(cat ".vagga/repo-with-suite/etc/apt/sources.list.d/8afb3430-xenial.list")
    [[ $repo_line = *" xenial universe" ]]

    run vagga _run repo-with-suite banner Wonderful
    [[ $output = "#     #"* ]]
}

@test "ubuntu trusty: faketime" {
    run vagga _build faketime
    [[ $output != *"shm_open:"* ]]
}
