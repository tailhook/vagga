setup() {
    cd /work/tests/alpine
}

@test "alpine: Alpine builds" {
    vagga _build v3.15
    link=$(readlink .vagga/v3.15)
    [[ $link = ".roots/v3.15.86210ade/root" ]]
}

@test "alpine: Check stdout" {
    run echo $(vagga v33-tar -cz vagga.yaml | tar -zt)
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/v33-tar)
    [[ $link = ".roots/v33-tar.ba02400b/root" ]]
    [[ $output = "vagga.yaml" ]]
}

@test "alpine: Check version" {
    run vagga _build alpine-check-version
    printf "%s\n" "${lines[@]}"
    [[ $status = 121 ]]
    [[ $output = *"Error checking alpine version"* ]]
}

@test "alpine: Run echo command" {
    run vagga echo-cmd hello
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = hello ]]
    run vagga echo-cmd world
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = world ]]
}

@test "alpine: Run bc on v3.4" {
    run vagga v34-calc 100*24
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/v34-calc)
    [[ $link = ".roots/v34-calc.519f0af9/root" ]]
}

@test "alpine: Run bc on v3.3" {
    run vagga v33-calc 100*24
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/v33-calc)
    [[ $link = ".roots/v33-calc.6d1f54ef/root" ]]
}

@test "alpine: Run bc on v3.2" {
    run vagga v32-calc 100*24
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "2400" ]]
    link=$(readlink .vagga/v32-calc)
    [[ $link = ".roots/v32-calc.a90f78f7/root" ]]
}

@test "alpine: Run bc on v3.15" {
    run vagga v3.15-calc 23*7+3
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "164" ]]
    link=$(readlink .vagga/v3.15-calc)
    [[ $link = ".roots/v3.15-calc.9826f0b8/root" ]]
}

@test "alpine: BuildDeps" {
    run vagga _build build-deps-with-version
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = *"480191"* ]]
    link=$(readlink .vagga/build-deps-with-version)
    [[ $link = ".roots/build-deps-with-version.5d980472/root" ]]

    run vagga _run build-deps-with-version bc
    printf "%s\n" "${lines[@]}"
    [[ $status = 124 ]]
}

@test "alpine: Run vagga inside alpine" {
    cp ../../vagga vagga_inside_alpine/
    cp ../../apk vagga_inside_alpine/
    cp ../../busybox vagga_inside_alpine/
    cp ../../alpine-keys.apk vagga_inside_alpine/

    run vagga vagga-alpine
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-2]} = 4eaca212 ]]
    [[ ${lines[${#lines[@]}-1]} = 4eaca212aa896f46307828745d58180d103f129545038b554912ee735cce20c9a710d3966bb5f9cabae28c8370effd13bf81e6b4af48d300dd3a2ee0c54bfbd7 ]]
}

@test "alpine: AlpineRepo minimal" {
    run vagga _build alpine-repo
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/alpine-repo)
    [[ $link = ".roots/alpine-repo.992729b6/root" ]]

    [[ $(tail -n 1 ".vagga/alpine-repo/etc/apk/repositories") = *"/v3.4/community" ]]
    repositories=($(sed "s/\/community/\/main/g" ".vagga/alpine-repo/etc/apk/repositories"))
    # test that additional repository has the same mirror
    [[ ${repositories[0]} = ${repositories[1]} ]]

    run vagga _run alpine-repo tini -h
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
}

@test "alpine: AlpineRepo full" {
    run vagga _build alpine-repo-full
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/alpine-repo-full)
    [[ $link = ".roots/alpine-repo-full.f703c75a/root" ]]

    [[ $(tail -n 1 ".vagga/alpine-repo-full/etc/apk/repositories") = \
        "@community http://dl-cdn.alpinelinux.org/alpine/edge/community" ]]

    run vagga _run alpine-repo-full tini -h
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
}

@test "alpine: Repo simple" {
    run vagga _build repo-simple
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/repo-simple)
    [[ $link = ".roots/repo-simple.9236cd3f/root" ]]

    [[ $(tail -n 1 ".vagga/repo-simple/etc/apk/repositories") = *"/v3.4/community" ]]
    repositories=($(sed "s/\/community/\/main/g" ".vagga/repo-simple/etc/apk/repositories"))
    # test that additional repository has the same mirror
    [[ ${repositories[0]} = ${repositories[1]} ]]

    run vagga _run repo-simple tini -h
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
}

@test "alpine: Repo with branch" {
    run vagga _build repo-with-branch
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/repo-with-branch)
    [[ $link = ".roots/repo-with-branch.9d184847/root" ]]

    [[ $(tail -n 1 ".vagga/repo-with-branch/etc/apk/repositories") = *"/edge/community" ]]
    repositories=($(sed "s/\/edge\/community/\/v3.4\/main/g" ".vagga/repo-with-branch/etc/apk/repositories"))
    # test that additional repository has the same mirror
    echo ${repositories[*]}
    [[ ${repositories[0]} = ${repositories[1]} ]]

    run vagga _run repo-with-branch tini -h
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
}

@test "alpine: Repo subcontainer" {
    run vagga _build repo-subcontainer
    printf "%s\n" "${lines[@]}"
    link=$(readlink .vagga/repo-subcontainer)
    [[ $link = ".roots/repo-subcontainer.64fdb8f2/root" ]]

    run vagga _run repo-subcontainer tini -h
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
}

@test "alpine: Run bc on v3.7" {
    run vagga v37-calc 50*12
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "600" ]]
    run vagga new-calc 51*13
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "663" ]]
    run vagga just-calc 53*14
    printf "%s\n" "${lines[@]}"
    [[ $status -eq 0 ]]
    [[ ${lines[${#lines[@]}-1]} = "742" ]]
    link=$(readlink .vagga/v37-calc)
    [[ $link = ".roots/v37-calc.ad28fb34/root" ]]
}

@test "alpine: descriptions" {
    run vagga
    [[ $status -eq 127 ]]
    printf "%s\n" "${lines[@]}"
    [[ "$output" = "Available commands:
    echo-cmd
    v3.15-calc
    v37-calc
                        (aliases: new-calc, just-calc)
    vagga-alpine

Old alpine commands:
    v32-calc
    v33-calc
    v33-tar
    v34-calc" ]]
}
