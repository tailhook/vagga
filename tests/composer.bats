setup() {
    cd /work/tests/composer
}

teardown() {
    cd /work/tests/composer
    if [ -d vendor ]; then rm -r vendor; fi
    if [ -f composer.lock ]; then rm composer.lock; fi
}

# php

@test "composer: php ubuntu trusty" {
    run vagga _run php-ubuntu-trusty php5 /composer/bin/laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-ubuntu-trusty)
    [[ $link = ".roots/php-ubuntu-trusty.0fd71d16/root" ]]
}

@test "composer: php ubuntu precise" {
    run vagga _run php-ubuntu-precise php5 /composer/bin/tester .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "No tests found" ]]
    link=$(readlink .vagga/php-ubuntu-precise)
    [[ $link = ".roots/php-ubuntu-precise.2ae4c071/root" ]]
}

@test "composer: php alpine 3.3" {
    run vagga _run php-alpine-3-3 php /composer/bin/laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-alpine-3-3)
    [[ $link = ".roots/php-alpine-3-3.471f38eb/root" ]]
}

@test "composer: php alpine 3.2" {
    run vagga _run php-alpine-3-2 php /composer/bin/laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-alpine-3-2)
    [[ $link = ".roots/php-alpine-3-2.e4506fdc/root" ]]
}

@test "composer: php ComposerRequirements" {
    run vagga _run php-composer-deps php /work/vendor/bin/laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/php-composer-deps)
    [[ $link = ".roots/php-composer-deps.89ed1ffe/root" ]]
}

@test "composer: php ComposerRequirements dev" {
    run vagga _run php-composer-dev-deps php /work/vendor/bin/laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    [[ -f vendor/nette/tester/composer.json ]]
    link=$(readlink .vagga/php-composer-dev-deps)
    [[ $link = ".roots/php-composer-dev-deps.0108a157/root" ]]

}

# hhvm

@test "composer: hhvm ubuntu trusty" {
    run vagga _run hhvm-ubuntu-trusty hhvm /composer/bin/laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/hhvm-ubuntu-trusty)
    [[ $link = ".roots/hhvm-ubuntu-trusty.b4c3809b/root" ]]
}

@test "composer: hhvm ubuntu precise" {
    run vagga _run hhvm-ubuntu-precise hhvm /composer/bin/tester .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "No tests found" ]]
    link=$(readlink .vagga/hhvm-ubuntu-precise)
    [[ $link = ".roots/hhvm-ubuntu-precise.2ae4c071/root" ]]
}

@test "composer: hhvm ComposerRequirements" {
    run vagga _run hhvm-composer-deps hhvm /work/vendor/bin/laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    link=$(readlink .vagga/hhvm-composer-deps)
    [[ $link = ".roots/hhvm-composer-deps.5ec27b91/root" ]]
}

@test "composer: hhvm ComposerRequirements dev" {
    run vagga _run hhvm-composer-dev-deps hhvm /work/vendor/bin/laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    [[ -f vendor/nette/tester/composer.json ]]
    link=$(readlink .vagga/hhvm-composer-dev-deps)
    [[ $link = ".roots/hhvm-composer-dev-deps.4699733f/root" ]]
}

@test "composer: hhvm alpine fail" {
    run vagga _build hhvm-alpine-fail
    printf "%s\n" "${lines[@]}"
    [[ "$output" = *"current linux distribution does not support features: [HHVM]"* ]]
    [[ $status = 121 ]]
    [[ ! -d .vagga/hhvm-alpine-fail ]]
}
