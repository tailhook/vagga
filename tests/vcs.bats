setup() {
    cd /work/tests/vcs
}

@test "vcs: urp from git checkout" {
    run vagga urp-git -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/git)
    [[ $link = ".roots/git.edc8d52e/root" ]]
}

@test "vcs: install from git checkout" {
    run vagga urp-git-install -Q key=val http://example.com
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/git-install)
    [[ $link = ".roots/git-install.1ced2199/root" ]]
}
