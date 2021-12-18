setup() {
    cd /work/tests/ubuntu
}

@test "Ubuntu builds" {
    vagga _build trusty
    link=$(readlink .vagga/trusty)
    [[ $link = ".roots/trusty.f0e3d303/root" ]]
}

@test "Ubuntu i386 builds" {
    vagga _build xenial-i386
    link=$(readlink .vagga/xenial-i386)
    [[ $link = ".roots/xenial-i386.30fb2ea2/root" ]]
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
    [[ $status = 0 ]]
    [[ $output = "hello" ]]
}

@test "Run absent command" {
    run vagga test something
    [[ $status -eq 121 ]]
    [[ $output =~ 'Command "test" not found and is not an alias' ]]
}

@test "Check arch support" {
    run vagga check-arch
    [[ $status = 0 ]]
    [[ $output = i386 ]]
}

@test "Run trusty bc" {
    run vagga trusty-calc 100*24
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/trusty-calc)
    [[ $link = ".roots/trusty-calc.010798c2/root" ]]
}

@test "Run xenial bc" {
    run vagga xenial-calc 23*7+3
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "164" ]]
    link=$(readlink .vagga/xenial-calc)
    [[ $link = ".roots/xenial-calc.321f6a11/root" ]]
}

@test "Test BuildDeps with version" {
    run vagga _build build-deps-with-version
    [[ $status = 0 ]]
    [[ $output = *"480191"* ]]
    link=$(readlink .vagga/build-deps-with-version)
    [[ $link = ".roots/build-deps-with-version.293fcc59/root" ]]

    run vagga _run build-deps-with-version bc
    [[ $status = 124 ]]
}

@test "Test focal universe" {
    run vagga _build ubuntu-universe
    [[ $status -eq 0 ]]
    link=$(readlink .vagga/ubuntu-universe)
    [[ $link = ".roots/ubuntu-universe.cf089a9f/root" ]]

    run vagga _run ubuntu-universe /usr/games/cowsay "Have you mooed today?"
    [[ $status = 0 ]]
    [[ $output = *"Have you mooed today?"* ]]
}

@test "Test VAGGAENV_* vars" {
    VAGGAENV_TESTVAR=testvalue run vagga _run trusty printenv TESTVAR
    [[ $status -eq 0 ]]
    [[ $output = testvalue ]]
}

@test "Test set env" {
    run vagga --environ TESTVAR=1value1 _run trusty printenv TESTVAR
    [[ $status -eq 0 ]]
    [[ $output = 1value1 ]]
}

@test "Test propagate env" {
    TESTVAR=2value2 run vagga --use-env TESTVAR _run trusty printenv TESTVAR
    [[ $status -eq 0 ]]
    [[ $output = 2value2 ]]
}

@test "The chfn just works (i.e. a no-op)" {
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
