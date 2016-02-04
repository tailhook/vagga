_vagga_completion() {
    cur="${COMP_WORDS[COMP_CWORD]}"
    COMPREPLY=( $(vagga _compgen "${COMP_WORDS[@]:1:$((COMP_CWORD-1))}" -- ${cur}) )
    return 0
}

complete -F _vagga_completion vagga
