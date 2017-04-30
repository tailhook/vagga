setup() {
    cd /work/tests/yarn
    rm yarn.lock || true
}

@test "yarn: " {
    cat <<END > package.json
{
  "devDependencies": {
    "resolve-cli": "0.1"
  }
}
END
    run vagga resolve .
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
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
}
