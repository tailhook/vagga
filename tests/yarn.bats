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
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = /work ]]
    run vagga _build pkg
    link=$(readlink .vagga/pkg)
    [[ $link =~ ^\.roots/pkg\..{8}/root ]]

    run vagga stat /usr/lib/node_modules/classnames
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
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-7]} = "  File: /usr/lib/node_modules/classnames" ]]
    link=$(readlink .vagga/pkg)
    echo "Link: $link"
    [[ $link =~ ^\.roots/pkg\..{8}/root ]]
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
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "1.0.1" ]]
    link=$(readlink .vagga/version)
    echo "Link: $link"
    [[ $link =~ ^\.roots/version\..{8}/root ]]
}
