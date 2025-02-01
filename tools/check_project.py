#!/usr/bin/env python3
import os
import json
from pathlib import Path

def check_project_structure(project_path):
    """Walk through the project directory and create a report of its structure."""
    structure = {
        "files": {},  # Will store file contents or sizes
        "directories": set(),  # Will store directory paths
        "empty_files": set(),  # Will store empty files
    }
    
    project_path = Path(project_path)
    for root, dirs, files in os.walk(project_path):
        rel_root = Path(root).relative_to(project_path)
        
        # Add directories
        for d in dirs:
            rel_path = str(rel_root / d)
            structure["directories"].add(rel_path)
            
        # Add files
        for f in files:
            file_path = Path(root) / f
            rel_path = str(rel_root / f)
            
            # Read file content for specific files
            if f in ["requirements.txt", "setup.py", "Dockerfile", "main.py"]:
                try:
                    with open(file_path, 'r') as file:
                        content = file.read()
                        if content.strip():
                            structure["files"][rel_path] = content
                        else:
                            structure["empty_files"].add(rel_path)
                except Exception as e:
                    structure["files"][rel_path] = f"Error reading file: {str(e)}"
            else:
                # Just store file size for other files
                size = os.path.getsize(file_path)
                if size > 0:
                    structure["files"][rel_path] = f"{size} bytes"
                else:
                    structure["empty_files"].add(rel_path)
    
    return structure

def print_report(structure):
    """Print a formatted report of the project structure."""
    print("\n=== Project Structure Report ===\n")
    
    print("Directories:")
    for d in sorted(structure["directories"]):
        print(f"  📁 {d}")
    
    print("\nFiles with content:")
    for f, content in sorted(structure["files"].items()):
        print(f"  📄 {f}")
        if isinstance(content, str) and not content.endswith('bytes'):
            print("    Content:")
            for line in content.split('\n')[:5]:  # Show first 5 lines
                print(f"      {line}")
            if len(content.split('\n')) > 5:
                print("      ...")
    
    print("\nEmpty files:")
    for f in sorted(structure["empty_files"]):
        print(f"  ❌ {f}")

if __name__ == "__main__":
    project_path = "/root/repos/build-system/build/test_project_001"
    structure = check_project_structure(project_path)
    print_report(structure)
