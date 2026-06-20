#!/usr/bin/env bash
# Bash completion for PrismNote

_prismnote_completion() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Main options
    opts="--help --version --port --data --host --log-level --config --plugins"

    case "${prev}" in
        --port)
            # Port number - no completion
            return 0
            ;;
        --data)
            # Directory path completion
            COMPREPLY=( $(compgen -d -- ${cur}) )
            return 0
            ;;
        --host)
            # Host address - no completion
            return 0
            ;;
        --log-level)
            COMPREPLY=( $(compgen -W "debug info warn error" -- ${cur}) )
            return 0
            ;;
        --config)
            # File path completion
            COMPREPLY=( $(compgen -f -- ${cur}) )
            return 0
            ;;
        --plugins)
            # Directory path completion
            COMPREPLY=( $(compgen -d -- ${cur}) )
            return 0
            ;;
        *)
            ;;
    esac

    # Complete options
    if [[ ${cur} == -* ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        return 0
    fi

    return 0
}

complete -F _prismnote_completion prismnote
