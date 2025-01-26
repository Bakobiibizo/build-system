# Documentation Engine Prompts

## Documentation Generation Prompt
```
You are generating documentation for a system component. Given:

[COMPONENT_DESCRIPTION]

Generate documentation that includes:
1. Component overview
2. Architecture details
3. Usage examples
4. API reference

Your response should be in markdown format:
```markdown
# Component Name

## Overview
[overview_text]

## Architecture
[architecture_details]

## Usage
[usage_examples]

## API Reference
[api_documentation]
```
```

## Progress Update Prompt
```
You are updating progress documentation. Given:

1. Current progress: [PROGRESS_DOC]
2. New changes: [CHANGES]
3. Timeline: [TIMELINE]

Update the progress document to:
1. Reflect new changes
2. Update status
3. Adjust timeline
4. Note dependencies

Your response should be in markdown format:
```markdown
# Progress Update

## Current Status
[status_update]

## Completed Items
[completed_items]

## Upcoming Tasks
[upcoming_tasks]

## Timeline
[updated_timeline]
```
```

## Documentation Review Prompt
```
You are reviewing system documentation. Given:

1. Documentation: [DOC_CONTENT]
2. Style guide: [STYLE_GUIDE]
3. Requirements: [REQUIREMENTS]

Review for:
1. Technical accuracy
2. Completeness
3. Style compliance
4. Clarity

Your response should be in JSON format:
{
    "technical_issues": ["issue_1"],
    "missing_content": ["missing_1"],
    "style_violations": ["violation_1"],
    "clarity_improvements": ["improvement_1"]
}
```

## Template Generation Prompt
```
You are creating a documentation template. Given:

1. Document type: [DOC_TYPE]
2. Required sections: [SECTIONS]
3. Style requirements: [STYLE_REQS]

Create a template that:
1. Includes all required sections
2. Follows style guidelines
3. Provides clear placeholders
4. Includes usage instructions

Your response should be in JSON format:
{
    "template_name": "string",
    "template_content": "string",
    "variables": ["var_1"],
    "usage_guide": "string"
}
```
