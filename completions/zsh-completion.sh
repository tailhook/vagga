#compdef vagga
#autoload

_cmds () {
    local cmds listopts

    # Show hidden options only when underscore is typed
    if [[ $words[2] = '_'* ]]; then
        listopts="--all"
    fi

    # Check if in folder with correct vagga.yaml file
    vagga _list 1>/dev/null 2>/dev/null
    if [ $? -eq 0 ]; then
        IFS=$'\n' cmds=($(vagga _list --zsh $listopts))
    else
        cmds=()
    fi
    _describe -t commands 'Available commands' cmds
}

_arguments -C -s "1: :_cmds" '*::arg:->args' --
case $state in
    (args)
        words[1]="vagga _help ${words[1]}"
        _arguments -C -s --
esac
