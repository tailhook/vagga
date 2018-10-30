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
    run vagga resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    run vagga _build pkg
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/pkg)
    [[ $link = ".roots/pkg.dfb332bf/root" ]]

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
    echo "Link: $link"
    [[ $link = ".roots/pkg.06364e8c/root" ]]
    run vagga resolve .
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "/work" ]]
}

@test "yarn: specific yarn version" {
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
    run vagga version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "1.0.1" ]]
    link=$(readlink .vagga/version)
    echo "Link: $link"
    [[ $link = ".roots/version.0a8ecf27/root" ]]
}
