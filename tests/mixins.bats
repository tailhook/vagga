setup() {
    cd /work/tests/mixins
}

@test "mixins: Runs original command" {
    run vagga _build alpine
    run vagga top
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = top ]]
}

@test "mixins: Overrides command" {
    run vagga _build alpine
    run vagga overrides
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = overrides ]]
}

@test "mixins: Overrides in the middle" {
    run vagga _build alpine
    run vagga m1x
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = m2 ]]
}

@test "mixins: Mixin command 1" {
    run vagga _build alpine
    run vagga m1
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = m1 ]]
}

@test "mixins: Mixin command 2" {
    run vagga _build alpine
    run vagga m2
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output = m2 ]]
}

@test "mixins: List is correct" {
    run vagga _build alpine
    run vagga _list
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ "${lines[0]}" = "m1                  " ]]
    [[ "${lines[1]}" = "m1x                 " ]]
    [[ "${lines[2]}" = "m2                  " ]]
    [[ "${lines[3]}" = "overrides           " ]]
    [[ "${lines[4]}" = "top                 " ]]
}

@test "mixins: Minimum version warning" {
    cd ../mixins_version
    run vagga _build test
    printf "%s\n" "${lines[@]}"
    [[ $status = 0 ]]
    [[ $output != *"UnknownBuildStep"* ]]
    [[ $output = *"Please upgrade vagga"* ]]
    [[ $(echo $output |grep "Minimum Vagga Error" |wc -l) = 1 ]]
}
