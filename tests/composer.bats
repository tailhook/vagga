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
    [[ $link = ".roots/composer-lifecycle.015a2c4d/root" ]]
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

@test "composer: php ubuntu trusty" {
    run vagga _run php-ubuntu-trusty laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-ubuntu-trusty)
    [[ $link = ".roots/php-ubuntu-trusty.341640f6/root" ]]
}

@test "composer: php ubuntu precise" {
    run vagga _run php-ubuntu-precise tester .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "No tests found" ]]
    link=$(readlink .vagga/php-ubuntu-precise)
    [[ $link = ".roots/php-ubuntu-precise.83e8d6e2/root" ]]
}

@test "composer: php alpine 3.3" {
    run vagga _run php-alpine-3-3 laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-alpine-3-3)
    [[ $link = ".roots/php-alpine-3-3.4bcc930b/root" ]]
}

@test "composer: php alpine 3.2" {
    run vagga _run php-alpine-3-2 laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-alpine-3-2)
    [[ $link = ".roots/php-alpine-3-2.6164c8bf/root" ]]
}

@test "composer: php ComposerDependencies" {
    run vagga _run php-composer-deps laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-composer-deps)
    [[ $link = ".roots/php-composer-deps.bbdd2965/root" ]]
}

@test "composer: php ComposerDependencies dev" {
    run vagga _run php-composer-dev-deps task greet
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello, Vagga!" ]]
    link=$(readlink .vagga/php-composer-dev-deps)
    [[ $link = ".roots/php-composer-dev-deps.0b5d7565/root" ]]
}

@test "composer: php ComposerDependencies dev ubuntu" {
    run vagga _run php-composer-dev-deps-ubuntu task greet
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello, Vagga!" ]]
    link=$(readlink .vagga/php-composer-dev-deps-ubuntu)
    [[ $link = ".roots/php-composer-dev-deps-ubuntu.ede6f799/root" ]]
}

@test "composer: php ComposerDependencies prefer dist" {
    run vagga _run php-composer-deps-prefer-dist task greet
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello, Vagga!" ]]
    link=$(readlink .vagga/php-composer-deps-prefer-dist)
    [[ $link = ".roots/php-composer-deps-prefer-dist.0b5d7565/root" ]]
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
    [[ $link = ".roots/php-composer-deps-lock.c4e5cc58/root" ]]
}

# hhvm

@test "composer: hhvm ubuntu trusty" {
    skip
    run vagga _run hhvm-ubuntu-trusty laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/hhvm-ubuntu-trusty)
    [[ $link = ".roots/hhvm-ubuntu-trusty.5222979f/root" ]]
}
