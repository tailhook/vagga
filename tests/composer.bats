setup() {
    cd /work/tests/composer
}

teardown() {
    cd /work/tests/composer
    if [ -f composer.lock ]; then rm composer.lock; fi
}

# test composer is available in PATH and removed after container is built
@test "composer: lifecycle" {
    run vagga _build composer-lifecycle
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = *"Composer version"* ]]
    [[ ! -f ".vagga/composer-lifecycle/usr/local/bin/composer" ]]
    link=$(readlink .vagga/composer-lifecycle)
    [[ $link = ".roots/composer-lifecycle.e9e6610b/root" ]]
}

@test "composer: keep composer after build" {
    run vagga _build keep-composer
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ -f .vagga/keep-composer/usr/local/bin/composer ]]
}

@test "composer: change vendor directory" {
    run vagga _build change-vendor-dir
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ -d .vagga/change-vendor-dir/usr/local/dependencies/vendor/nette/tester ]]
}

# php

@test "composer: php ubuntu xenial" {
    run vagga _run php-ubuntu-xenial laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-ubuntu-xenial)
    [[ $link = ".roots/php-ubuntu-xenial.87465c41/root" ]]
}

@test "composer: php ubuntu trusty" {
    run vagga _run php-ubuntu-trusty laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-ubuntu-trusty)
    [[ $link = ".roots/php-ubuntu-trusty.ed35312d/root" ]]
}

@test "composer: php ubuntu precise" {
    run vagga _run php-ubuntu-precise tester .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "No tests found" ]]
    link=$(readlink .vagga/php-ubuntu-precise)
    [[ $link = ".roots/php-ubuntu-precise.57872fd3/root" ]]
}

@test "composer: php alpine 3.4" {
    run vagga _run php-alpine-3_4 laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-alpine-3_4)
    [[ $link = ".roots/php-alpine-3_4.730f7f8f/root" ]]
}

@test "composer: php alpine 3.3" {
    run vagga _run php-alpine-3_3 laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-alpine-3_3)
    [[ $link = ".roots/php-alpine-3_3.2a51fce5/root" ]]
}

@test "composer: php alpine 3.2" {
    run vagga _run php-alpine-3_2 laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-alpine-3_2)
    [[ $link = ".roots/php-alpine-3_2.5153e63b/root" ]]
}

@test "composer: php ComposerDependencies" {
    run vagga _run php-composer-deps laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-composer-deps)
    [[ $link = ".roots/php-composer-deps.e581a959/root" ]]
}

@test "composer: php ComposerDependencies ubuntu xenial" {
    run vagga _run php-composer-deps-ubuntu-xenial laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-composer-deps-ubuntu-xenial)
    [[ $link = ".roots/php-composer-deps-ubuntu-xenial.e3a858bd/root" ]]
}

@test "composer: php ComposerDependencies ubuntu trusty" {
    run vagga _run php-composer-deps-ubuntu-trusty laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-composer-deps-ubuntu-trusty)
    [[ $link = ".roots/php-composer-deps-ubuntu-trusty.592bc51f/root" ]]
}

@test "composer: php ComposerDependencies dev" {
    run vagga _run php-composer-dev-deps task greet
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello, Vagga!" ]]
    link=$(readlink .vagga/php-composer-dev-deps)
    [[ $link = ".roots/php-composer-dev-deps.5f703887/root" ]]
}

@test "composer: php ComposerDependencies dev ubuntu xenial" {
    run vagga _run php-composer-dev-deps-ubuntu-xenial task greet
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello, Vagga!" ]]
    link=$(readlink .vagga/php-composer-dev-deps-ubuntu-xenial)
    [[ $link = ".roots/php-composer-dev-deps-ubuntu-xenial.d4b00831/root" ]]
}

@test "composer: php ComposerDependencies dev ubuntu trusty" {
    run vagga _run php-composer-dev-deps-ubuntu-trusty task greet
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello, Vagga!" ]]
    link=$(readlink .vagga/php-composer-dev-deps-ubuntu-trusty)
    [[ $link = ".roots/php-composer-dev-deps-ubuntu-trusty.fe7ec5ad/root" ]]
}

@test "composer: php ComposerDependencies prefer dist" {
    run vagga _run php-composer-deps-prefer-dist task greet
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello, Vagga!" ]]
    link=$(readlink .vagga/php-composer-deps-prefer-dist)
    [[ $link = ".roots/php-composer-deps-prefer-dist.5f703887/root" ]]
}

@test "composer: php ComposerDependencies wrong prefer" {
    run vagga _build php-composer-deps-wrong-prefer
    printf "%s\n" "${lines[@]}"
    [[ $status = 121 ]]
    [[ $output = *"Value of 'ComposerDependencies.prefer' must be either 'source' or 'dist', 'wrong' given"* ]]
}

@test "composer: php ComposerDependencies lock" {
    cd /work/tests/composer_lock
    run vagga _run php-composer-deps-lock laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = *"The lock file is not up to date with the latest changes in composer.json"* ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-composer-deps-lock)
    [[ $link = ".roots/php-composer-deps-lock.b416bf95/root" ]]
}

# hhvm

@test "composer: hhvm ubuntu xenial" {
    run vagga _run hhvm-ubuntu-xenial laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/hhvm-ubuntu-xenial)
    [[ $link = ".roots/hhvm-ubuntu-xenial.189813c0/root" ]]
}
