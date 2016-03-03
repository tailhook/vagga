setup() {
    cd /work/tests/composer
}

teardown() {
    cd /work/tests/composer
    if [ -d vendor ]; then rm -r vendor; fi
    if [ -f composer.lock ]; then rm composer.lock; fi
    cd /work/tests/composer_lock
    if [ -d vendor ]; then rm -r vendor; fi
}

# php

@test "composer: php ubuntu trusty" {
    run vagga _run php-ubuntu-trusty laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-ubuntu-trusty)
    [[ $link = ".roots/php-ubuntu-trusty.0fd71d16/root" ]]
}

@test "composer: php ubuntu precise" {
    skip
    run vagga _run php-ubuntu-precise tester .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "No tests found" ]]
    link=$(readlink .vagga/php-ubuntu-precise)
    [[ $link = ".roots/php-ubuntu-precise.2ae4c071/root" ]]
}

@test "composer: php alpine 3.3" {
    run vagga _run php-alpine-3-3 laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-alpine-3-3)
    [[ $link = ".roots/php-alpine-3-3.471f38eb/root" ]]
}

@test "composer: php alpine 3.2" {
    run vagga _run php-alpine-3-2 laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-alpine-3-2)
    [[ $link = ".roots/php-alpine-3-2.e4506fdc/root" ]]
}

@test "composer: php ComposerDependencies" {
    run vagga _run php-composer-deps laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-composer-deps)
    [[ $link = ".roots/php-composer-deps.89ed1ffe/root" ]]
}

@test "composer: php ComposerDependencies dev" {
    run vagga _run php-composer-dev-deps laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    # [[ -f /usr/local/lib/composer/vendor/nette/tester/composer.json ]]
    link=$(readlink .vagga/php-composer-dev-deps)
    [[ $link = ".roots/php-composer-dev-deps.0108a157/root" ]]
}

@test "composer: php ComposerDependencies prefer dist" {
    run vagga _run php-composer-deps-prefer-dist laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-composer-deps-prefer-dist)
    [[ $link = ".roots/php-composer-deps-prefer-dist.0108a157/root" ]]
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
    # [[ -f /usr/local/lib/composer/vendor/nette/tester/composer.json ]]
    link=$(readlink .vagga/php-composer-deps-lock)
    [[ $link = ".roots/php-composer-deps-lock.a6277bfe/root" ]]

}

# hhvm

@test "composer: hhvm ubuntu trusty" {
    run vagga _run hhvm-ubuntu-trusty laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/hhvm-ubuntu-trusty)
    [[ $link = ".roots/hhvm-ubuntu-trusty.7619aac5/root" ]]
}

@test "composer: hhvm ComposerDependencies" {
    run vagga _run hhvm-composer-deps laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/hhvm-composer-deps)
    [[ $link = ".roots/hhvm-composer-deps.57f7b1e2/root" ]]
}

@test "composer: hhvm ComposerDependencies dev" {
    run vagga _run hhvm-composer-dev-deps laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    # [[ -f /usr/local/lib/composer/vendor/nette/tester/composer.json ]]
    link=$(readlink .vagga/hhvm-composer-dev-deps)
    [[ $link = ".roots/hhvm-composer-dev-deps.eb3c0986/root" ]]
}

@test "composer: hhvm ComposerDependencies lock" {
    cd /work/tests/composer_lock
    run vagga _run hhvm-composer-deps-lock laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = *"The lock file is not up to date with the latest changes in composer.json"* ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    # [[ -f /usr/local/lib/composer/vendor/nette/tester/composer.json ]]
    link=$(readlink .vagga/hhvm-composer-deps-lock)
    [[ $link = ".roots/hhvm-composer-deps-lock.b587d75f/root" ]]
}
