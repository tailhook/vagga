setup() {
    cd /work/tests/uidmap
}

@test "uidmap: Too much uids" {
    run vagga _build too-much-uids
    printf "%s\n" "${lines[@]}"
    echo "Status: $status"
    [[ $status = 121 ]]
    [[ $output =~ "Number of allowed subuids is too small" ]]
}

@test "uidmap: Too much gids" {
    run vagga _build too-much-gids
    printf "%s\n" "${lines[@]}"
    echo "Status: $status"
    [[ $status = 121 ]]
    [[ $output =~ "Number of allowed subgids is too small" ]]
}

@test "uidmap: Bad subuid" {
    echo user:0:65535 > /tmp/subuid
    echo root:x:0:0::/root:/bin/sh > /tmp/passwd
    echo user:!:1000:100::/home/user:/bin/sh >> /tmp/passwd
    mount --bind /tmp/passwd /etc/passwd
    mount --bind /tmp/subuid /etc/subuid
    run su user -c "vagga --ignore-owner-check _clean"
    umount /etc/subuid
    umount /etc/passwd
    printf "%s\n" "${lines[@]}"
    echo "Status: $status"
    [[ $status = 121 ]]
    [[ $output =~ "includes original id" ]]
}

@test "uidmap: Bad subgid" {
    echo user:0:65536 > /tmp/subgid
    echo root:x:0:0::/root:/bin/sh > /tmp/passwd
    echo user:!:1000:100::/home/user:/bin/sh >> /tmp/passwd
    mount --bind /tmp/passwd /etc/passwd
    mount --bind /tmp/subgid /etc/subgid
    run su user -c "vagga --ignore-owner-check _build too-much-uids"
    umount /etc/subgid
    umount /etc/passwd
    printf "%s\n" "${lines[@]}"
    echo "Status: $status"
    [[ $status = 121 ]]
    [[ $output =~ "includes original id" ]]
}
