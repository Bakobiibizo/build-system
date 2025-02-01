# Project Configuration Template

## Overview
This document defines the comprehensive template for generating project configurations in our build system.

## Guidelines

### 1. Project Naming and Description
- Choose a clear, descriptive project name
- Provide a concise but informative project description
- Ensure the name is kebab-case or snake_case

### 2. Technology Stack Selection
- Select appropriate language and framework based on project requirements
- Consider scalability, performance, and ecosystem support
- Prefer modern, well-maintained technologies

### 3. Dependency Management
- Include essential production dependencies
- Add development and tooling dependencies
- Specify precise, compatible version constraints
- Prioritize security, performance, and community support

### 4. Directory Structure
- Create a clean, scalable project layout
- Separate concerns with logical directory divisions
- Include standard directories: src, tests, config, docs
- Create module-specific subdirectories as needed

### 5. Build and Development Configuration
- Define build tool and build scripts
- Include development, testing, and production scripts
- Configure linting, formatting, and code quality tools

### 6. Initialization and Recommendations
- Provide initial setup commands
- Include best practice recommendations
- Consider future extensibility

## JSON Configuration Schema

```json
{
  "project_name": "kebab-case-or-snake_case",
  "description": "Concise project description",
  "project_type": "WebApplication|CLI|Library|Microservice",
  "language": "Rust|Python|TypeScript|Go",
  "framework": "Rocket|Actix|FastAPI|Django|Next.js",
  "technologies": ["list", "of", "key", "technologies"],
  "dependencies": {
    "production": {"package": "version"},
    "development": {"package": "version"}
  },
  "build_config": {
    "build_tool": "cargo|npm|poetry",
    "scripts": {
      "dev": "development start command",
      "build": "build command",
      "test": "test command",
      "lint": "linting command"
    }
  },
  "directory_structure": {
    "src": ["main.rs", "modules/", "utils/"],
    "tests": ["integration/", "unit/"],
    "config": ["app.toml", "database.toml"]
  },
  "initialization_commands": [
    "initial setup commands",
    "dependency installation"
  ],
  "recommendations": [
    "architectural best practices",
    "potential improvements"
  ]
}
```

## Constraints
- Respond ONLY with a valid, complete JSON configuration
- Ensure all fields are present and meaningful
- Validate JSON structure before responding
