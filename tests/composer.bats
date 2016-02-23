setup() {
    cd /work/tests/composer
}

teardown() {
    cd /work/tests/composer
    if [ -d vendor ]; then rm -r vendor; fi
    if [ -f composer.lock ]; then rm composer.lock; fi
}

@test "composer: ubuntu pkg" {
    run vagga _run pkg-ubuntu php5 /composer/bin/laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/pkg-ubuntu)
    [[ $link = ".roots/pkg-ubuntu.0fd71d16/root" ]]
}

@test "composer: precise pkg" {
    run vagga _run pkg-precise php5 /composer/bin/tester .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "No tests found" ]]
    link=$(readlink .vagga/pkg-precise)
    [[ $link = ".roots/pkg-precise.2ae4c071/root" ]]
}

@test "composer: alpine pkg" {
    run vagga _run pkg-alpine php /composer/bin/laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/pkg-alpine)
    [[ $link = ".roots/pkg-alpine.471f38eb/root" ]]
}

@test "composer: ComposerDependencies" {
    run vagga _run composer-deps php /work/vendor/bin/laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/composer-deps)
    [[ $link = ".roots/composer-deps.89ed1ffe/root" ]]
}
@test "composer: ComposerDependencies dev" {
    run vagga _run composer-dev-deps php /work/vendor/bin/laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    [[ -f vendor/nette/tester/composer.json ]]
    link=$(readlink .vagga/composer-dev-deps)
    [[ $link = ".roots/composer-dev-deps.0108a157/root" ]]

}
