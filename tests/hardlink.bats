setup() {
    cd /work/tests/hardlink
}

@test "hardlinking" {
    rm -rf .vagga
    export VAGGA_SETTINGS="
        index-all-images: true
        hard-link-identical-files: true
    "

    run vagga _build hello
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/hello)
    [[ $link = ".roots/hello.0ae0aab6/root" ]]

    run vagga _build hello-and-bye
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/hello-and-bye)
    [[ $link = ".roots/hello-and-bye.84b3175b/root" ]]
    # There are 2 extra hardlinks because of /etc/resolv.conf & /etc/hosts
    [[ $output = *"Found and linked 3 (12B) identical files"* ]]

    sed -i 's/!/?/' .vagga/hello/etc/hello.txt
    run vagga _build --force hello-and-bye
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/hello-and-bye)
    [[ $link = ".roots/hello-and-bye.84b3175b/root" ]]
    # There are 2 extra hardlinks because of /etc/resolv.conf & /etc/hosts
    [[ $output = *"Found and linked 2 (0B) identical files"* ]]
    [[ $(cat .vagga/hello/etc/hello.txt) = "Hello world?" ]]
    [[ $(cat .vagga/hello-and-bye/etc/hello.txt) = "Hello world!" ]]
}

@test "hardlinking between projects" {
    rm -rf .storage
    mkdir .storage
    export VAGGA_SETTINGS="
        storage-dir: /work/tests/hardlink/.storage
        index-all-images: true
        hard-link-identical-files: true
        hard-link-between-projects: true
    "

    cd /work/tests/hardlink/project-1
    rm -rf .vagga
    run vagga _build hello
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/hello)
    [[ $link = ".lnk/.roots/hello.0ae0aab6/root" ]]

    cd /work/tests/hardlink/project-2
    rm -rf .vagga
    run vagga _build hello-and-bye
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/hello-and-bye)
    [[ $link = ".lnk/.roots/hello-and-bye.84b3175b/root" ]]
    [[ $output = *"Found and linked 3 (12B) identical files"* ]]
}

@test "hardlink cmd" {
    rm -rf .vagga
    run vagga _build --force hello
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/hello)
    [[ $link = ".roots/hello.0ae0aab6/root" ]]

    run vagga _hardlink
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = *"Found and linked 0"* ]]

    run vagga _build --force hello-and-bye
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/hello-and-bye)
    [[ $link = ".roots/hello-and-bye.84b3175b/root" ]]

    run vagga _hardlink
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = *"Found and linked 3 (12B) identical files"* ]]

    [[ $(stat -c "%i" .vagga/hello/etc/hello.txt) = \
        $(stat -c "%i" .vagga/hello-and-bye/etc/hello.txt) ]]

    [[ $(stat -c "%h" .vagga/hello-and-bye/etc/hello.txt) = 2 ]]
    [[ $(stat -c "%h" .vagga/hello-and-bye/etc/bye.txt) = 1 ]]

    sed -i 's/!/?/' .vagga/hello/etc/hello.txt
    vagga _build --force hello-and-bye
    run vagga _hardlink
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/hello-and-bye)
    [[ $link = ".roots/hello-and-bye.84b3175b/root" ]]
    [[ $output = *"Found and linked 2 (0B) identical files"* ]]
    [[ $(cat .vagga/hello/etc/hello.txt) = "Hello world?" ]]
    [[ $(cat .vagga/hello-and-bye/etc/hello.txt) = "Hello world!" ]]
}

@test "hardlink global" {
    rm -rf .storage
    mkdir .storage
    export VAGGA_SETTINGS="
        storage-dir: /work/tests/hardlink/.storage
    "

    cd /work/tests/hardlink/project-1
    rm -rf .vagga
    run vagga _build --force hello
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/hello)
    [[ $link = ".lnk/.roots/hello.0ae0aab6/root" ]]

    run vagga _build --force hi
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/hi)
    [[ $link = ".lnk/.roots/hi.079e4655/root" ]]

    run vagga _hardlink --global
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = *"Found and linked 2 (0B) identical files"* ]]

    cd /work/tests/hardlink/project-2
    rm -rf .vagga
    run vagga _build --force hello-and-bye
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    link=$(readlink .vagga/hello-and-bye)
    [[ $link = ".lnk/.roots/hello-and-bye.84b3175b/root" ]]

    run vagga _hardlink --global
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = *"Found and linked 3 (12B) identical files"* ]]
}

@test "verify cmd" {
    vagga _build --force hello
    vagga _build --force hello-and-bye
    vagga _hardlink

    run vagga _verify hello
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]

    echo "Hi!" > .vagga/hello-and-bye/etc/hello.txt
    touch .vagga/hello-and-bye/etc/bonjour.txt
    rm .vagga/hello-and-bye/etc/bye.txt

    run vagga _verify hello-and-bye
    printf "%s\n" "${lines[@]}"
    [[ $status = 1 ]]
    [[ $output = *"Container is corrupted"* ]]
    [[ $output = *"Missing"*"/etc/bye.txt"*"Extra"*"/etc/bonjour.txt"*"Corrupted"*"/etc/hello.txt" ]]

    run vagga _verify hello
    printf "%s\n" "${lines[@]}"
    [[ $status = 1 ]]
    [[ $output = *"Container is corrupted"* ]]
    [[ $output = *"Corrupted"*"/etc/hello.txt"* ]]
    [[ $output != *"Missing"* ]]
    [[ $output != *"Extra"* ]]
}
