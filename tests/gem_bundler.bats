setup() {
    cd /work/tests/gem_bundler
}

teardown() {
    cd /work/tests/gem_bundler
    if [ -f Gemfile.lock ]; then rm Gemfile.lock; fi
    if [ -d .bundle ]; then rm -r .bundle; fi
}

@test "gem/bundler: alpine pkg" {
    run vagga _run pkg-alpine rake --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "rake, version 11.1.1" ]]
    link=$(readlink .vagga/pkg-alpine)
    [[ $link = ".roots/pkg-alpine.d41880cf/root" ]]
}

@test "gem/bundler: alpine pkg no update gem" {
    run vagga _run pkg-alpine-no-update-gem rake --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "rake, version 11.1.1" ]]
    link=$(readlink .vagga/pkg-alpine-no-update-gem)
    [[ $link = ".roots/pkg-alpine-no-update-gem.ecca9ae2/root" ]]
}

@test "gem/bundler: ubuntu focal pkg" {
    run vagga _run pkg-ubuntu-focal fpm --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "1.14.1" ]]
    link=$(readlink .vagga/pkg-ubuntu-focal)
    [[ $link = ".roots/pkg-ubuntu-focal.476ebe37/root" ]]
}

@test "gem/bundler: ubuntu focal pkg no update gem" {
    run vagga _run pkg-ubuntu-focal-no-update-gem fpm --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "1.14.1" ]]
    link=$(readlink .vagga/pkg-ubuntu-focal-no-update-gem)
    [[ $link = ".roots/pkg-ubuntu-focal-no-update-gem.8639e43f/root" ]]
}

@test "gem/bundler: ubuntu bionic pkg" {
    run vagga _run pkg-ubuntu-bionic fpm --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "1.14.1" ]]
    link=$(readlink .vagga/pkg-ubuntu-bionic)
    [[ $link = ".roots/pkg-ubuntu-bionic.9927a372/root" ]]
}

@test "gem/bundler: ubuntu bionic pkg no update gem" {
    run vagga _run pkg-ubuntu-bionic-no-update-gem fpm --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "1.14.1" ]]
    link=$(readlink .vagga/pkg-ubuntu-bionic-no-update-gem)
    [[ $link = ".roots/pkg-ubuntu-bionic-no-update-gem.c8a1d390/root" ]]
}

@test "gem/bundler: alpine GemBundle" {
    run vagga _run bundle-alpine fpm --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "1.14.1" ]]
    [[ -d .vagga/bundle-alpine/usr/lib/ruby/gems/2.3.0/gems/cuba-3.9.3 ]]
    [[ -d .vagga/bundle-alpine/usr/lib/ruby/gems/2.3.0/gems/fpm-1.14.1 ]]
    link=$(readlink .vagga/bundle-alpine)
    [[ $link = ".roots/bundle-alpine.47d4da43/root" ]]
}

@test "gem/bundler: alpine GemBundle without dev" {
    run vagga _build bundle-alpine-no-dev
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ -d .vagga/bundle-alpine-no-dev/usr/lib/ruby/gems/2.3.0/gems/cuba-3.9.3 ]]
    [[ ! -d .vagga/bundle-alpine-no-dev/usr/lib/ruby/gems/2.3.0/gems/fpm-1.14.1 ]]
    link=$(readlink .vagga/bundle-alpine-no-dev)
    [[ $link = ".roots/bundle-alpine-no-dev.47d4da43/root" ]]
}

@test "gem/bundler: ubuntu GemBundle" {
    run vagga _run bundle-ubuntu fpm --version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "1.14.1" ]]
    [[ -d .vagga/bundle-ubuntu/var/lib/gems/2.7.0/gems/cuba-3.9.3 ]]
    [[ -d .vagga/bundle-ubuntu/var/lib/gems/2.7.0/gems/fpm-1.14.1 ]]
    link=$(readlink .vagga/bundle-ubuntu)
    [[ $link = ".roots/bundle-ubuntu.b224ee75/root" ]]
}

@test "gem/bundler: ubuntu GemBundle without dev" {
    run vagga _build bundle-ubuntu-no-dev
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ -d .vagga/bundle-ubuntu-no-dev/var/lib/gems/2.7.0/gems/cuba-3.9.3 ]]
    [[ ! -d .vagga/bundle-ubuntu-no-dev/var/lib/gems/2.7.0/gems/fpm-1.14.1 ]]
    link=$(readlink .vagga/bundle-ubuntu-no-dev)
    [[ $link = ".roots/bundle-ubuntu-no-dev.b224ee75/root" ]]
}

@test "gem/bundler: GemBundle invalid trust_policy" {
    run vagga _build bundle-invalid-trust-policy
    printf "%s\n" "${lines[@]}"
    [[ $status = 121 ]]
    [[ $output = *"Value of 'GemBundle.trust_policy' must be 'LowSecurity', 'MediumSecurity' or 'HighSecurity', 'invalid' given"* ]]
}
