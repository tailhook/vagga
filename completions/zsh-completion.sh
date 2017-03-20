#compdef vagga
#autoload

VAGGA=${VAGGA:-vagga}

_list () {
    local cmds listopts

    # Show hidden options only when underscore is typed
    if [[ $words[2] = '_'* ]]; then
        listopts="--all"
    fi

    # Check if in folder with correct vagga.yaml file
    $VAGGA _list 1>/dev/null 2>/dev/null
    if [ $? -eq 0 ]; then
        IFS=$'\n' cmds=($($VAGGA _list "$1" $listopts))
    else
        cmds=()
    fi
    _describe -t commands 'Available commands' cmds
}

_arguments -C -s "1: :{_list --zsh}" '*::arg:->args' --
case $state in
    (args)
        cmd=${words[1]}
        if [[ ${cmd} = "_run" || $cmd = "_build" ]] then;
            _arguments -C -s "1: :{_list --containers}"
        else
            words[1]="$VAGGA _help ${cmd}"
            _arguments -C -s --
        fi
esac
