setup() {
    cd /work/tests/vcs
}

@test "vcs: urp from git checkout" {
    run vagga urp-git -Q key=val http://example.com
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/git)
    [[ $link = ".roots/git.71287cc7/root" ]]
}

@test "vcs: install from git checkout" {
    run vagga urp-git-install -Q key=val http://example.com
    [[ $status = 0 ]]
    [[ ${lines[${#lines[@]}-1]} = http://example.com?key=val ]]
    link=$(readlink .vagga/git-install)
    [[ $link = ".roots/git-install.4e1d802a/root" ]]
}

@test "vcs: git describe" {
    rm -rf .git
    git init
    git config user.email test@example.com
    git config user.name Test
    git add vagga.yaml
    git commit -m "Initial commit"
    git tag -a v0.0.1 -m "Test tag"
    git describe

    run vagga _build git-describe-no-file
    [[ $status = 0 ]]

    run vagga _build git-describe
    [[ $status = 0 ]]
    link=$(readlink .vagga/git-describe)
    [[ $link = ".roots/git-describe.022317fb/root" ]]
    [[ $(cat .vagga/git-describe/version.txt) = "v0.0.1" ]]

    touch test.txt
    git add -f test.txt
    git commit -m "Test commit"
    git describe

    run vagga _build git-describe
    [[ $status = 0 ]]
    new_link=$(readlink .vagga/git-describe)
    [[ $link != $new_link ]]
    [[ $(cat .vagga/git-describe/version.txt) = "v0.0.1-1-"* ]]

    git add .gitignore
    git commit -m ".gitignore added"
    git tag -a ignoreme0.0.1 -m "ignore me"
    git describe

    run vagga _build git-describe-pattern
    [[ $status = 0 ]]
    [[ $(cat .vagga/git-describe-pattern/version.txt) = "v0.0.1-2-"* ]]
}
