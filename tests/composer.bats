setup() {
    cd /work/tests/composer
}

teardown() {
    cd /work/tests/composer
    rm -f composer.lock
}

# test composer is available in PATH and removed after container is built
@test "composer: lifecycle" {
    run vagga _build composer-lifecycle
    [[ $status = 0 ]]
    [[ $output = *"Composer version"* ]]
    [[ ! -f ".vagga/composer-lifecycle/usr/local/bin/composer" ]]
    link=$(readlink .vagga/composer-lifecycle)
    [[ $link = ".roots/composer-lifecycle.160821b4/root" ]]
}

@test "composer: keep composer after build" {
    run vagga _build keep-composer
    [[ $status = 0 ]]
    [[ -f .vagga/keep-composer/usr/local/bin/composer ]]
}

@test "composer: change vendor directory" {
    run vagga _build change-vendor-dir
    [[ $status = 0 ]]
    [[ -d .vagga/change-vendor-dir/usr/local/dependencies/vendor/nette/tester ]]
}

# php

@test "composer: php ubuntu xenial" {
    run vagga _run php-ubuntu-xenial laravel --version
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer 1.3.0" ]]
    link=$(readlink .vagga/php-ubuntu-xenial)
    [[ $link = ".roots/php-ubuntu-xenial.2533125c/root" ]]
}

@test "composer: php ubuntu trusty" {
    run vagga _run php-ubuntu-trusty laravel --version
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer 1.3.0" ]]
    link=$(readlink .vagga/php-ubuntu-trusty)
    [[ $link = ".roots/php-ubuntu-trusty.6f3ec930/root" ]]
}

@test "composer: php ubuntu focal" {
    run vagga _run php-ubuntu-focal tester .
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "No tests found" ]]
    link=$(readlink .vagga/php-ubuntu-focal)
    [[ $link = ".roots/php-ubuntu-focal.00877c3c/root" ]]
}

@test "composer: php alpine 3.5" {
    run vagga _run php-alpine-3_5 laravel --version
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer 1.3.0" ]]
    link=$(readlink .vagga/php-alpine-3_5)
    [[ $link = ".roots/php-alpine-3_5.9a42a892/root" ]]
}

@test "composer: php alpine 3.5 php7" {
    run vagga _run php-alpine-3_5-php7 php --version
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-3]} = "PHP 7.0."*" (cli)"* ]]

    run vagga _run php-alpine-3_5-php7 laravel --version
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer 1.3.0" ]]

    link=$(readlink .vagga/php-alpine-3_5-php7)
    [[ $link = ".roots/php-alpine-3_5-php7.8147eafd/root" ]]
}

@test "composer: php alpine 3.4" {
    run vagga _run php-alpine-3_4 laravel --version
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer 1.3.0" ]]
    link=$(readlink .vagga/php-alpine-3_4)
    [[ $link = ".roots/php-alpine-3_4.ea0d9f02/root" ]]
}

@test "composer: php alpine 3.3" {
    run vagga _run php-alpine-3_3 laravel --version
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer 1.3.0" ]]
    link=$(readlink .vagga/php-alpine-3_3)
    [[ $link = ".roots/php-alpine-3_3.11d9ad5c/root" ]]
}

@test "composer: php alpine 3.2" {
    run vagga _run php-alpine-3_2 laravel --version
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer 1.3.0" ]]
    link=$(readlink .vagga/php-alpine-3_2)
    [[ $link = ".roots/php-alpine-3_2.14b46cd2/root" ]]
}

@test "composer: php ComposerDependencies alpine 3.5" {
    run vagga _run php-composer-deps-alpine-3_5 laravel --version
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-composer-deps-alpine-3_5)
    [[ $link = ".roots/php-composer-deps-alpine-3_5.6c42be15/root" ]]
}

@test "composer: php ComposerDependencies alpine 3.5 php7" {
    run vagga _run php-alpine-3_5-php7 php --version
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-3]} = "PHP 7.0."*" (cli)"* ]]

    run vagga _run php-composer-deps-alpine-3_5-php7 laravel --version
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]

    link=$(readlink .vagga/php-composer-deps-alpine-3_5-php7)
    [[ $link = ".roots/php-composer-deps-alpine-3_5-php7.4cbe8516/root" ]]
}

@test "composer: php ComposerDependencies alpine 3.4" {
    run vagga _run php-composer-deps laravel --version
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-composer-deps)
    [[ $link = ".roots/php-composer-deps.1fb5bd2b/root" ]]
}

@test "composer: php ComposerDependencies ubuntu xenial" {
    run vagga _run php-composer-deps-ubuntu-xenial laravel --version
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-composer-deps-ubuntu-xenial)
    [[ $link = ".roots/php-composer-deps-ubuntu-xenial.cb7ffde8/root" ]]
}

@test "composer: php ComposerDependencies ubuntu trusty" {
    run vagga _run php-composer-deps-ubuntu-trusty laravel --version
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-composer-deps-ubuntu-trusty)
    [[ $link = ".roots/php-composer-deps-ubuntu-trusty.48543817/root" ]]
}

@test "composer: php ComposerDependencies dev" {
    run vagga _run php-composer-dev-deps task greet
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello, Vagga!" ]]
    link=$(readlink .vagga/php-composer-dev-deps)
    [[ $link = ".roots/php-composer-dev-deps.1fb5bd2b/root" ]]
}

@test "composer: php ComposerDependencies dev ubuntu xenial" {
    run vagga _run php-composer-dev-deps-ubuntu-xenial task greet
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello, Vagga!" ]]
    link=$(readlink .vagga/php-composer-dev-deps-ubuntu-xenial)
    [[ $link = ".roots/php-composer-dev-deps-ubuntu-xenial.cb7ffde8/root" ]]
}

@test "composer: php ComposerDependencies dev ubuntu trusty" {
    run vagga _run php-composer-dev-deps-ubuntu-trusty task greet
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello, Vagga!" ]]
    link=$(readlink .vagga/php-composer-dev-deps-ubuntu-trusty)
    [[ $link = ".roots/php-composer-dev-deps-ubuntu-trusty.48543817/root" ]]
}

@test "composer: php ComposerDependencies prefer dist" {
    run vagga _run php-composer-deps-prefer-dist task greet
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Hello, Vagga!" ]]
    link=$(readlink .vagga/php-composer-deps-prefer-dist)
    [[ $link = ".roots/php-composer-deps-prefer-dist.1fb5bd2b/root" ]]
}

@test "composer: php ComposerDependencies wrong prefer" {
    run vagga _build php-composer-deps-wrong-prefer
    [[ $status = 121 ]]
    [[ $output = *"Value of 'ComposerDependencies.prefer' must be either 'source' or 'dist', 'wrong' given"* ]]
}

@test "composer: php ComposerDependencies lock" {
    cd /work/tests/composer_lock
    run vagga _run php-composer-deps-lock laravel --version
    [[ $status = 0 ]]
    [[ $output = *"The lock file is not up to date with the latest changes in composer.json"* ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-composer-deps-lock)
    [[ $link = ".roots/php-composer-deps-lock.1fb10887/root" ]]
}

# hhvm

@test "composer: hhvm ubuntu xenial" {
    skip "Something wrong with default hhvm package on ubuntu"
    run vagga _run hhvm-ubuntu-xenial laravel --version
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer 1.3.0" ]]
    link=$(readlink .vagga/hhvm-ubuntu-xenial)
    [[ $link = ".roots/hhvm-ubuntu-xenial.1664cbd2/root" ]]
}
