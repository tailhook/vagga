setup() {
    cd /work/tests/yarn
    rm yarn.lock || true
}

@test "yarn: minimal" {
    cat <<END > package.json
{
  "devDependencies": {
    "resolve-cli": "0.1"
  }
}
END
    # don't check bin installations, because recent yarn have them broken
    # run vagga resolve .
    # printf "%s\n" "${lines[@]}"
    # [[ $status = 0 ]]
    # [[ ${lines[${#lines[@]}-1]} = /work ]]
    run vagga _build pkg
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/pkg)
    [[ $link = ".roots/pkg.06f039be/root" ]]

    run vagga stat /usr/lib/node_modules/classnames
    printf "%s\n" "${lines[@]}"
    [[ $status = 1 ]]

    cat <<END > package.json

{
  "dependencies": {
    "classnames": "2.2"
  },
  "devDependencies": {
    "resolve-cli": "0.1"
  }
}
END
    run vagga stat /usr/lib/node_modules/classnames
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-7]} = "  File: /usr/lib/node_modules/classnames" ]]
    link=$(readlink .vagga/pkg)
    [[ $link = ".roots/pkg.1a63a916/root" ]]
    run vagga _run pkg resolve .
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "/work" ]]
}

@test "yarn: specific yarn version" {
    run vagga version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "1.0.1" ]]
    link=$(readlink .vagga/version)
    [[ $link = ".roots/version.58b6add6/root" ]]
}
