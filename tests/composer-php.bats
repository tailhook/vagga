setup() {
    cd /work/tests/composer
}

@test "composer: ubuntu pkg" {
    run vagga _run pkg-ubuntu php5 /composer/bin/task
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "No Taskfile found" ]]
    link=$(readlink .vagga/pkg-ubuntu)
    [[ $link = ".roots/pkg-ubuntu.87aba8b8/root" ]]
}

@test "composer: precise pkg" {
    skip
    run vagga _run pkg-precise php5 /composer/bin/laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/pkg-precise)
    [[ $link = ".roots/pkg-precise.47a114e9/root" ]]
}

@test "composer: alpine pkg" {
    skip
    run vagga _run pkg-alpine php /composer/bin/laravel --version
    # printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/pkg-alpine)
    [[ $link = ".roots/pkg-alpine.864640c4/root" ]]
}

@test "composer: ubuntu git" {
    skip
    run vagga _run git-ubuntu php5 /composer/bin/laravel --version
    # printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/git-ubuntu)
    [[ $link = ".roots/git-ubuntu.a3bf710f/root" ]]
}

@test "composer: alpine git" {
    skip
    run vagga _run git-alpine php /composer/bin/laravel --version
    # printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/git-alpine)
    [[ $link = ".roots/git-alpine.d6b3d182/root" ]]
}

@test "composer: ComposerDependencies" {
    skip
    run vagga _run composer-deps php5 /composer/bin/laravel --version
    # printf "%s\n" "${lines[@]}"
    [[ $status = 124 ]]  # no resolve but has classnames --v
    [[ -f .vagga/composer-deps/usr/lib/node_modules/classnames/index.js ]]
    link=$(readlink .vagga/composer-deps)
    [[ $link = ".roots/composer-deps.22206178/root" ]]
}
@test "composer: ComposerDependencies dev" {
    skip
    run vagga _run composer-dev-deps php5 /composer/bin/laravel --version
    # printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    link=$(readlink .vagga/composer-dev-deps)
    [[ $link = ".roots/composer-dev-deps.1dd280f9/root" ]]
}
