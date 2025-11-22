
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
            [CompletionResult]::new('list-courses', 'list-courses', [CompletionResultType]::ParameterValue, 'List courses')
            [CompletionResult]::new('list-assessments', 'list-assessments', [CompletionResultType]::ParameterValue, 'List assessments')
            [CompletionResult]::new('upload-assessment', 'upload-assessment', [CompletionResultType]::ParameterValue, 'Upload assessment')
            [CompletionResult]::new('login', 'login', [CompletionResultType]::ParameterValue, 'Login to Crowdmark')
            break
        }
        'climark;list-courses' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'f')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'format')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Don''t print error messages')
            [CompletionResult]::new('--silent', '--silent', [CompletionResultType]::ParameterName, 'Don''t print error messages')
            break
        }
        'climark;list-assessments' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Hide scores')
            [CompletionResult]::new('--hide-scores', '--hide-scores', [CompletionResultType]::ParameterName, 'Hide scores')
            [CompletionResult]::new('-j', '-j', [CompletionResultType]::ParameterName, 'Print in JSON Format')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Print in JSON Format')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Don''t print error messages')
            [CompletionResult]::new('--silent', '--silent', [CompletionResultType]::ParameterName, 'Don''t print error messages')
            break
        }
        'climark;upload-assessment' {
            [CompletionResult]::new('--silent', '--silent', [CompletionResultType]::ParameterName, 'Don''t print error messages')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Submit assignment after upload')
            [CompletionResult]::new('--submit', '--submit', [CompletionResultType]::ParameterName, 'Submit assignment after upload')
            break
        }
        'climark;login' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
