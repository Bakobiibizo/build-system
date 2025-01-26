# CLI Prompts

## Command Analysis Prompt
```
You are analyzing a CLI command. Given:

[COMMAND_INPUT]

Analyze the command for:
1. Command structure
2. Option validation
3. Parameter requirements
4. Execution requirements

Your response should be in JSON format:
{
    "command_type": "string",
    "options": {
        "option_name": "value"
    },
    "validation_errors": [],
    "execution_plan": ["step_1"]
}
```

## Help Generation Prompt
```
You are generating help documentation for a CLI command. Given:

1. Command name: [COMMAND]
2. Command options: [OPTIONS]
3. Usage patterns: [PATTERNS]

Generate help text that includes:
1. Command description
2. Option details
3. Usage examples
4. Common patterns

Your response should be in markdown format:
```markdown
# Command: [name]

## Description
[description_text]

## Options
[options_list]

## Usage
[usage_examples]

## Examples
[example_commands]
```
```

## Error Message Generation Prompt
```
You are generating an error message for a CLI error. Given:

1. Error type: [ERROR_TYPE]
2. Error context: [CONTEXT]
3. User command: [COMMAND]

Create an error message that:
1. Clearly states the problem
2. Suggests solutions
3. Provides context
4. Includes next steps

Your response should be in JSON format:
{
    "error_message": "string",
    "error_details": "string",
    "suggested_actions": ["action_1"],
    "help_resources": ["resource_1"]
}
```

## Output Formatting Prompt
```
You are formatting CLI output. Given:

1. Command output: [OUTPUT]
2. Format type: [FORMAT]
3. Verbosity level: [VERBOSITY]

Format the output to:
1. Match format requirements
2. Respect verbosity
3. Highlight important info
4. Maintain readability

Your response should be in JSON format:
{
    "formatted_output": "string",
    "highlights": ["highlight_1"],
    "summary": "string",
    "details": ["detail_1"]
}
```
