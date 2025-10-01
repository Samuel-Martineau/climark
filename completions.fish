set -l commands list-courses list-assessments
complete -c climark -f
complete -c climark -n "not __fish_seen_subcommand_from $commands" \
    -a "$commands"

complete -c climark -n "__fish_seen_subcommand_from list-assessments" -a "(climark list-courses --format=plain)"