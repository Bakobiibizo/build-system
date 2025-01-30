# Build System: AI-Driven Project Generation

## Overview

Build System is an advanced CLI tool that leverages AI to dynamically generate project configurations and documentation. It provides an intelligent workflow for creating new software projects with minimal manual configuration.

## Features

- AI-Powered Project Generation
- Dynamic Template Management
- Intelligent Configuration Suggestions
- Automated Documentation

## Project Generation Workflow

### Quick Start

```bash
# Generate a new project
cargo run -- generate --name my-awesome-project --description "A web application for task management"
```

### Templates

Project templates are stored in `.reference/templates/`. Each template is a JSON configuration that defines:
- Project structure
- Technology stack
- Architectural components
- Deployment strategies

### Customization

1. Create a new template in `.reference/templates/`
2. Specify technologies, frameworks, and features
3. Use AI to refine and enhance project configuration

### Example Template

```json
{
    "project_name": "web_app_template",
    "technologies": ["rust", "actix-web", "react"],
    "architecture": {
        "backend": "actix-web",
        "frontend": "react"
    }
}
```

## CLI Usage

### Generate Command

```bash
# Basic generation
cargo run -- generate --name project-name

# With description
cargo run -- generate --name project-name --description "Detailed project requirements"

# Specify template
cargo run -- generate --name project-name --template web_app
```

## Architecture

- `src/cli/`: CLI command handling
- `src/prompt/`: AI-driven prompt management
- `src/doc/`: Documentation generation
- `.reference/templates/`: Project configuration templates

## Contributing

1. Add new project templates
2. Improve AI configuration generation
3. Enhance documentation workflows

## Requirements

- Rust 1.75+
- OpenAI API Key (for AI-powered generation)

## License

MIT License
