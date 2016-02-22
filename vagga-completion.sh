_vagga_completion() {
    cur="${COMP_WORDS[COMP_CWORD]}"
    COMPREPLY=( $(vagga _compgen "${COMP_WORDS[@]:1:$((COMP_CWORD-1))}" -- ${cur}) )
    if [[ ${COMPREPLY} == "" ]]; then
        COMPREPLY=( $(compgen -f -- ${cur}) )
    fi
    return 0
}

complete -o filenames -F _vagga_completion vagga
