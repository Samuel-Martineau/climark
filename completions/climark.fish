# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_climark_global_optspecs
	string join \n crowdmark-session-token=
end

function __fish_climark_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_climark_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_climark_using_subcommand
	set -l cmd (__fish_climark_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c climark -n "__fish_climark_needs_command" -l crowdmark-session-token -r
complete -c climark -n "__fish_climark_needs_command" -f -a "list-courses" -d 'List courses'
complete -c climark -n "__fish_climark_needs_command" -f -a "list-assessments" -d 'List assessments'
complete -c climark -n "__fish_climark_needs_command" -f -a "upload-assessment" -d 'Upload assessment'
complete -c climark -n "__fish_climark_needs_command" -f -a "login" -d 'Login to Crowdmark'
complete -c climark -n "__fish_climark_using_subcommand list-courses" -s f -l format -r -f -a "pretty\t''
plain\t''
json\t''"
complete -c climark -n "__fish_climark_using_subcommand list-courses" -s s -l silent -d 'Don\'t print error messages'
complete -c climark -n "__fish_climark_using_subcommand list-assessments" -s h -l hide-scores -d 'Hide scores'
complete -c climark -n "__fish_climark_using_subcommand list-assessments" -s j -l json -d 'Print in JSON Format'
complete -c climark -n "__fish_climark_using_subcommand list-assessments" -s s -l silent -d 'Don\'t print error messages'
complete -c climark -n "__fish_climark_using_subcommand upload-assessment" -l silent -d 'Don\'t print error messages'
complete -c climark -n "__fish_climark_using_subcommand upload-assessment" -s s -l submit -d 'Submit assignment after upload'

complete -c climark -f
complete -c climark -kn "__fish_seen_subcommand_from list-assessments" -a "(climark list-courses -s --format=plain)"
