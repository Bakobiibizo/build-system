# AI-Powered Project Configuration Generation

## Iterative Prompt

Generate and refine a project configuration through multiple iterations.

### Basic Usage
```bash
build-system inference iterative-prompt \
    --initial-prompt "Create a web application project" \
    --max-iterations 3 \
    --refinement-instruction "Enhance the configuration with modern best practices and scalability"
```

### Save to File
```bash
build-system inference iterative-prompt \
    --initial-prompt "Create a data science project" \
    --max-iterations 2 \
    --refinement-instruction "Add more comprehensive machine learning libraries" \
    --output project_config.json
```

## Conditional Check

Generate a project configuration with conditional logic.

### Basic Usage
```bash
build-system inference conditional-check \
    --initial-prompt "Create a data science project" \
    --condition-prompt "Check if advanced machine learning capabilities are required" \
    --option-a-prompt "Add advanced ML libraries and frameworks" \
    --option-b-prompt "Use basic data processing tools"
```

### Save to File
```bash
build-system inference conditional-check \
    --initial-prompt "Create a web application project" \
    --condition-prompt "Determine if the project needs microservices architecture" \
    --option-a-prompt "Design a microservices-based architecture" \
    --option-b-prompt "Create a monolithic web application" \
    --output project_config.json
```

## Tips
- Use descriptive and specific prompts for best results
- Experiment with different iteration counts and refinement instructions
- The output can be saved to a file for later use or further processing
