# Prompt Storage CLI

## Overview
The Prompt Storage CLI allows you to manage and interact with prompt configurations using a persistent key-value storage system.

## Commands

### Store a Prompt
```bash
build-system prompt-storage store --input prompt.json [--namespace prompt]
```
- Stores a JSON-formatted prompt
- Generates a unique UUID for the prompt
- Optional namespace (default: 'prompt')

### Retrieve a Prompt
```bash
build-system prompt-storage retrieve --id <UUID> [--namespace prompt]
```
- Retrieves a prompt by its UUID
- Optional namespace (default: 'prompt')

### List Prompts
```bash
build-system prompt-storage list [--namespace prompt]
```
- Lists all prompts in a given namespace
- Optional namespace (default: 'prompt')

### Delete a Prompt
```bash
build-system prompt-storage delete --id <UUID> [--namespace prompt]
```
- Deletes a prompt by its UUID
- Optional namespace (default: 'prompt')

### Validate JSON
```bash
build-system prompt-storage validate --input prompt.json --schema schema.json
```
- Validates a JSON file against a JSON schema
- Useful for checking prompt configurations

## Example Workflow
```bash
# Validate a prompt against its schema
build-system prompt-storage validate \
    --input examples/web_app_prompt.json \
    --schema examples/web_app_prompt_schema.json

# Store the prompt
build-system prompt-storage store \
    --input examples/web_app_prompt.json \
    --namespace web-prompts

# List stored prompts
build-system prompt-storage list --namespace web-prompts

# Retrieve a specific prompt
build-system prompt-storage retrieve \
    --id <GENERATED_UUID> \
    --namespace web-prompts
```

## Storage Location
Prompts are stored in:
- Linux/Mac: `~/.build-system/prompt_storage`
- Windows: `%USERPROFILE%\.build-system\prompt_storage`
