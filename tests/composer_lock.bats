setup() {
    cd /work/tests/composer_lock
}

teardown() {
    cd /work/tests/composer_lock
    if [ -d vendor ]; then rm -r vendor; fi
}

# php

@test "composer: php ComposerDependencies dev" {
    run vagga _run php-composer-dev-deps php /work/vendor/bin/laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = *"The lock file is not up to date with the latest changes in composer.json"* ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    [[ -f vendor/nette/tester/composer.json ]]
    link=$(readlink .vagga/php-composer-dev-deps)
    [[ $link = ".roots/php-composer-dev-deps.a6277bfe/root" ]]

}

# hhvm

@test "composer: hhvm ComposerDependencies dev" {
    run vagga _run hhvm-composer-dev-deps hhvm /work/vendor/bin/laravel --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = *"The lock file is not up to date with the latest changes in composer.json"* ]]
    [[ ${lines[${#lines[@]}-1]} = "Laravel Installer version 1.3.0" ]]
    [[ -f vendor/nette/tester/composer.json ]]
    link=$(readlink .vagga/hhvm-composer-dev-deps)
    [[ $link = ".roots/hhvm-composer-dev-deps.d7a4a267/root" ]]
}
