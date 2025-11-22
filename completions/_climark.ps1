
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'climark' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'climark'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'climark' {
            [CompletionResult]::new('--crowdmark-session-token', '--crowdmark-session-token', [CompletionResultType]::ParameterName, 'crowdmark-session-token')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('list-courses', 'list-courses', [CompletionResultType]::ParameterValue, 'list-courses')
            [CompletionResult]::new('list-assessments', 'list-assessments', [CompletionResultType]::ParameterValue, 'list-assessments')
            [CompletionResult]::new('upload-assessment', 'upload-assessment', [CompletionResultType]::ParameterValue, 'upload-assessment')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'climark;list-courses' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'f')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'format')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 's')
            [CompletionResult]::new('--silent', '--silent', [CompletionResultType]::ParameterName, 'silent')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'climark;list-assessments' {
            [CompletionResult]::new('-j', '-j', [CompletionResultType]::ParameterName, 'j')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'json')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 's')
            [CompletionResult]::new('--silent', '--silent', [CompletionResultType]::ParameterName, 'silent')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'climark;upload-assessment' {
            [CompletionResult]::new('--silent', '--silent', [CompletionResultType]::ParameterName, 'silent')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 's')
            [CompletionResult]::new('--submit', '--submit', [CompletionResultType]::ParameterName, 'submit')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'climark;help' {
            [CompletionResult]::new('list-courses', 'list-courses', [CompletionResultType]::ParameterValue, 'list-courses')
            [CompletionResult]::new('list-assessments', 'list-assessments', [CompletionResultType]::ParameterValue, 'list-assessments')
            [CompletionResult]::new('upload-assessment', 'upload-assessment', [CompletionResultType]::ParameterValue, 'upload-assessment')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'climark;help;list-courses' {
            break
        }
        'climark;help;list-assessments' {
            break
        }
        'climark;help;upload-assessment' {
            break
        }
        'climark;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
