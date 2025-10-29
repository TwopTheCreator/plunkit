def handle_devrc_command(self, tokens: List[str]):
        """Handle .devrc specific commands"""
        i = 0
        while i < len(tokens):
            token = tokens[i]
            
            if token == '-out' and i + 1 < len(tokens):
                self.output_to_file(tokens[i + 1])
                i += 2
            
            elif token == '-crfolder' and i + 1 < len(tokens):
                self.create_folder(tokens[i + 1])
                i += 2
            
            elif token == '-pop':
                if i + 1 < len(tokens):
                    print(f"✓ Pop operation: {tokens[i + 1]}")
                i += 2
            
            elif token == '-plugin':
                print("✓ Plugin mode enabled")
                i += 1
            
            elif token == '-config':
                print("✓ Config mode enabled")
                i += 1
            
            elif token == '-c':
                print("✓ Compile mode enabled")
                i += 1
            
            elif token == '-timed':
                print("✓ Timed operation enabled")
                i += 1
            
            elif token == '-mode' and i + 1 < len(tokens):
                print(f"✓ Mode set to: {tokens[i + 1]}")
                i += 2
            
            elif token == '-force':
                print("✓ Force mode enabled")
                i += 1
            
            elif token == '-a':
                print("✓ Append operation")
                i += 1
            
            elif token == '-locate' and i + 1 < len(tokens):
                print(f"✓ Locate: {tokens[i + 1]}")
                i += 2
            
            elif token == '-to':
                print("✓ Transform operation")
                i += 1
            
            elif token == '-ext' and i + 1 < len(tokens):
                print(f"✓ Extension: {tokens[i + 1]}")
                i += 2
            
            elif token == '-cmdbin':
                print("✓ Command binary mode")
                i += 1
            
            elif token == '-cmdline':
                print("✓ Command line mode")
                i += 1
            
            elif token == '-rline' and i + 1 < len(tokens):
                print(f"✓ Run line: {tokens[i + 1]}")
                i += 2
            
            elif token == '-r' and i + 1 < len(tokens):
                print(f"✓ Run mode: {tokens[i + 1]}")
                i += 2
            
            elif token == '-byp':
                print("✓ Bypass mode enabled")
                i += 1
            
            elif token == '-h' and i + 1 < len(tokens):
                print(f"✓ Handle pattern: {tokens[i + 1]}")
                i += 2
            
            elif token == '-ch':
                print("✓ Chain operation")
                i += 1
            
            elif token == '-numline':
                print("✓ Number line mode")
                i += 1
            
            elif token == '-ff':
                print("✓ Fast forward mode")
                i += 1
            
            elif token == '-glob' and i + 1 < len(tokens):
                print(f"✓ Glob pattern: {tokens[i + 1]}")
                i += 2
            
            elif token == '-set':
                print("✓ Set operation")
                i += 1
            
            elif token == '-getline':
                print("✓ Get line operation")
                i += 1
            
            elif token == '-linenum':
                print("✓ Line number operation")
                i += 1
            
            elif token == '-activeline':
                print("✓ Active line mode")
                i += 1
            
            elif token == '-enable':
                print("✓ Enable flag")
                i += 1
            
            elif token == '-commitline':
                print("✓ Commit line operation")
                i += 1
            
            else:
                i += 1#!/usr/bin/env python3
"""
DevRC DSL Interpreter
Parses and executes .devrc configuration files
"""

import re
import os
import json
import subprocess
from pathlib import Path
from typing import Dict, List, Any, Optional


class DevRCInterpreter:
    def __init__(self):
        self.variables = {}
        self.sections = {}
        self.section_types = {}
        self.current_section = None
        self.imported_files = set()
        self.import_stack = []
        self.environments = {}
        self.active_environment = None
        self.root_dir = os.getcwd()
        
    def parse_file(self, filepath: str) -> Dict[str, List[str]]:
        """Parse a .devrc file into sections"""
        # Prevent circular imports
        abs_path = os.path.abspath(filepath)
        if abs_path in self.import_stack:
            print(f"✗ Circular import detected: {filepath}")
            return {}
        
        self.import_stack.append(abs_path)
        
        with open(filepath, 'r') as f:
            content = f.read()
        
        sections = {}
        current_section = None
        current_type = None
        
        for line in content.split('\n'):
            original_line = line
            
            # Check for environment activation BEFORE removing comments
            if line.strip().startswith('#[') and ']/ACTIVATE' in line:
                self.handle_environment_activation(line.strip())
                continue
            
            # Check for inline imports with @DEVRC.IMPORT=
            if '@DEVRC.IMPORT=' in line:
                self.handle_inline_import(line)
                # Don't skip the line, let it be processed
            
            # Remove comments (but not environment markers)
            if '#' in line and not line.strip().startswith('#['):
                line = line.split('#')[0]
            
            line = line.strip()
            if not line:
                continue
            
            # Check for imports
            if line.startswith('@DEVRC.IMPORT.'):
                self.handle_import(line, os.path.dirname(filepath))
                continue
            
            # Check for type annotations
            if line.startswith('@[') and line.endswith(']'):
                current_type = line[2:-1]
                print(f"✓ Type annotation found: {current_type}")
                continue
                
            # Check for section headers
            if line.startswith('[') and line.endswith(']'):
                current_section = line[1:-1]
                sections[current_section] = []
                if current_type:
                    self.section_types[current_section] = current_type
                    current_type = None
            elif current_section:
                sections[current_section].append(line)
        
        self.import_stack.pop()
        return sections
    
    def tokenize(self, line: str) -> List[str]:
        """Tokenize a line into components"""
        # Handle quoted strings
        tokens = []
        current = []
        in_quotes = False
        
        for char in line:
            if char == '"':
                in_quotes = not in_quotes
                current.append(char)
            elif char in [' ', '\t'] and not in_quotes:
                if current:
                    tokens.append(''.join(current))
                    current = []
            else:
                current.append(char)
        
        if current:
            tokens.append(''.join(current))
        
        return tokens
    
    def parse_assignment(self, line: str) -> Optional[tuple]:
        """Parse variable assignment"""
        if '=' in line:
            parts = line.split('=', 1)
            var_name = parts[0].strip()
            var_value = parts[1].strip()
            return (var_name, var_value)
        return None
    
    def evaluate_expression(self, expr: str) -> Any:
        """Evaluate an expression"""
        expr = expr.strip()
        
        # Remove quotes
        if expr.startswith('"') and expr.endswith('"'):
            return expr[1:-1]
        
        # Check if it's a variable reference
        if expr in self.variables:
            return self.variables[expr]
        
        # Check for boolean literals
        if expr.lower() in ['true', '-true']:
            return True
        if expr.lower() in ['false', '-false']:
            return False
        
        # Check for null
        if expr.lower() == 'null':
            return None
        
        return expr
    
    def handle_import(self, line: str, base_path: str):
        """Handle @DEVRC.IMPORT.[variablename] statements"""
        # Parse import statement: @DEVRC.IMPORT.[variablename] or @DEVRC.IMPORT.[variablename]="path"
        match = re.match(r'@DEVRC\.IMPORT\.(\w+)(?:="?([^"]+)"?)?', line)
        if not match:
            print(f"✗ Invalid import syntax: {line}")
            return
        
        var_name = match.group(1)
        import_path = match.group(2)
        
        # If no path specified, check if variable exists
        if not import_path:
            if var_name not in self.variables:
                print(f"✗ Import failed: variable '{var_name}' not defined")
                return
            import_path = self.variables[var_name]
        
        # Resolve relative paths
        if not os.path.isabs(import_path):
            import_path = os.path.join(base_path, import_path)
        
        # Check if already imported
        abs_import_path = os.path.abspath(import_path)
        if abs_import_path in self.imported_files:
            print(f"✓ Already imported: {import_path}")
            return
        
        # Check if file exists
        if not os.path.exists(import_path):
            print(f"✗ Import file not found: {import_path}")
            return
        
        print(f"✓ Importing from: {import_path}")
        self.imported_files.add(abs_import_path)
        
        # Parse and merge the imported file
        imported_sections = self.parse_file(import_path)
        for section_name, lines in imported_sections.items():
            if section_name in self.sections:
                # Merge with existing section
                print(f"  ↳ Merging section: [{section_name}]")
                self.sections[section_name].extend(lines)
            else:
                # Add new section
                print(f"  ↳ Adding section: [{section_name}]")
                self.sections[section_name] = lines
                # Copy type if exists
                if section_name in self.section_types:
                    self.section_types[section_name] = self.section_types[section_name]
    
    def handle_inline_import(self, line: str):
        """Handle inline @DEVRC.IMPORT= statements within expressions"""
        # Parse: @DEVRC.IMPORT="./file.devrc" or @DEVRC.IMPORT="dirlist"
        match = re.search(r'@DEVRC\.IMPORT="([^"]+)"', line)
        if match:
            import_ref = match.group(1)
            print(f"✓ Inline import reference: {import_ref}")
            
            # Store as variable for later use
            if 'is STR' in line:
                var_match = re.search(r'is STR "([^"]+)"', line)
                if var_match:
                    var_name = var_match.group(1)
                    self.variables[var_name] = import_ref
                    print(f"  ↳ Stored as: {var_name}")
    
    def handle_environment_activation(self, line: str):
        """Handle #[environmentname]/ACTIVATE directives"""
        # Parse: #[environmentname]/ACTIVATE
        match = re.match(r'#\[([^\]]+)\]/ACTIVATE', line)
        if not match:
            print(f"✗ Invalid environment activation syntax: {line}")
            return
        
        env_name = match.group(1)
        
        # Create environment directory path from root
        env_path = os.path.join(self.root_dir, env_name)
        
        # Create environment if it doesn't exist
        if not os.path.exists(env_path):
            try:
                os.makedirs(env_path, exist_ok=True)
                print(f"✓ Created environment directory: {env_path}")
            except Exception as e:
                print(f"✗ Failed to create environment directory: {e}")
                return
        
        # Activate the environment
        self.active_environment = env_name
        self.environments[env_name] = {
            'path': env_path,
            'root': self.root_dir,
            'activated_at': os.getcwd(),
            'mode': 'SCRIPT',
            'exported': {},
            'subenv': {},
            'content': {}
        }
        
        # Set environment variable
        self.variables['env'] = env_name
        self.variables['activate'] = f"{env_name}/ACTIVATE"
        self.variables['currentdir'] = env_path
        
        # Change to environment directory
        try:
            os.chdir(env_path)
            print(f"✓ Environment activated: {env_name}")
            print(f"  ↳ Working directory: {env_path}")
            print(f"  ↳ Root directory: {self.root_dir}")
            print(f"  ↳ Mode: SCRIPT")
        except Exception as e:
            print(f"✗ Failed to change to environment directory: {e}")
    
    def create_folder(self, path: str):
        """Create a folder if it doesn't exist"""
        path = path.strip('"').replace('*', '')
        try:
            Path(path).mkdir(parents=True, exist_ok=True)
            print(f"✓ Created folder: {path}")
        except Exception as e:
            print(f"✗ Error creating folder {path}: {e}")
    
    def output_to_file(self, path: str, content: Any = None):
        """Handle output to file"""
        path = path.strip('"').replace('*', '')
        try:
            if '*' in path or path.endswith('/'):
                # Directory output
                Path(path).mkdir(parents=True, exist_ok=True)
                print(f"✓ Prepared output directory: {path}")
            else:
                # File output
                Path(path).parent.mkdir(parents=True, exist_ok=True)
                if content:
                    with open(path, 'w') as f:
                        if isinstance(content, dict):
                            json.dump(content, f, indent=2)
                        else:
                            f.write(str(content))
                print(f"✓ Output to: {path}")
        except Exception as e:
            print(f"✗ Error outputting to {path}: {e}")
    
    def execute_command(self, command: List[str]):
        """Execute a system command"""
        try:
            result = subprocess.run(command, capture_output=True, text=True)
            print(f"✓ Executed: {' '.join(command)}")
            return result.stdout
        except Exception as e:
            print(f"✗ Error executing command: {e}")
            return None
    
    def process_line(self, line: str):
        """Process a single line of DevRC code"""
        tokens = self.tokenize(line)
        if not tokens:
            return
        
        # Handle special comment directives (xcn-byp)
        if line.strip().startswith('#xcn-byp'):
            self.handle_xcn_bypass(line)
            return
        
        # Handle assignments with special syntax
        assignment = self.parse_assignment(line)
        if assignment:
            var_name, var_value = assignment
            
            # Handle special assignments like poot={}
            if var_value.strip() in ['{}', '[]']:
                self.variables[var_name] = {}
                print(f"✓ Initialized {var_name} as empty container")
                return
            
            # Handle complex assignments with try()
            if 'try (' in var_value:
                self.handle_try_assignment(var_name, var_value)
                return
            
            # Handle function assignments
            if var_value.strip().startswith('function ('):
                self.handle_function_assignment(var_name, var_value)
                return
            
            self.variables[var_name] = self.evaluate_expression(var_value)
            print(f"✓ Set {var_name} = {self.variables[var_name]}")
            return
        
        # Handle dirlist with -glob
        if tokens[0] == 'dirlist' or 'dirlist=' in line:
            self.handle_dirlist(line)
            return
        
        # Handle currentdir
        if tokens[0] == 'currentdir' or 'currentdir=' in line:
            self.handle_currentdir(line)
            return
        
        # Handle subenv
        if tokens[0] == 'subenv' or 'subenv=' in line:
            self.handle_subenv(line)
            return
        
        # Handle prod/dev/debug environments
        if tokens[0] in ['prod', 'dev', 'debug'] and '=' in line:
            self.handle_environment_category(line)
            return
        
        # Handle linenum
        if tokens[0] == 'linenum' or 'linenum=' in line:
            self.handle_linenum(line)
            return
        
        # Handle current with activeline
        if tokens[0] == 'current' or 'current=' in line:
            self.handle_current_line(line)
            return
        
        # Handle function definitions
        if tokens[0] == 'function':
            self.handle_function_definition(line)
            return
        
        # Handle return statements
        if tokens[0] == 'return':
            self.handle_return_statement(line)
            return
        
        # Handle export statements
        if tokens[0] == 'export':
            self.handle_export_statement(line)
            return
        
        # Handle activate keyword
        if tokens[0] == 'activate':
            self.handle_activate_keyword(line)
            return
        
        # Handle .devrc commands
        if tokens[0] == '.devrc':
            self.handle_devrc_command(tokens[1:])
        
        # Handle if statements
        elif tokens[0] == 'if':
            self.handle_if_statement(line)
        
        # Handle for loops
        elif tokens[0] == 'for':
            self.handle_for_loop(line)
        
        # Handle do statements
        elif tokens[0] == 'do':
            self.handle_do_statement(tokens[1:])
        
        # Handle out command
        elif tokens[0] == 'out':
            if len(tokens) > 1:
                self.output_to_file(tokens[1])
        
        # Handle get operations
        elif tokens[0] == 'get':
            self.handle_get_operation(line)
        
        # Handle in operations  
        elif tokens[0] == 'in':
            self.handle_in_operation(line)
        
        # Handle try blocks
        elif tokens[0] == 'try':
            self.handle_try_block(line)
    
    def handle_function_definition(self, line: str):
        """Handle function definitions"""
        match = re.search(r'function\s+(\w+)\s*\(', line)
        if match:
            func_name = match.group(1)
            print(f"✓ Function defined: {func_name}")
            self.variables[func_name] = "function"
        else:
            # Anonymous function or function call syntax
            print(f"✓ Function block defined")
    
    def handle_return_statement(self, line: str):
        """Handle return statements"""
        # Extract return value
        match = re.search(r'return\s+(.+)', line)
        if match:
            return_val = match.group(1).strip()
            print(f"✓ Return: {return_val}")
            if self.active_environment:
                self.environments[self.active_environment]['return_value'] = return_val
    
    def handle_export_statement(self, line: str):
        """Handle export statements for environment variables"""
        # Parse: export name ( ... )
        match = re.search(r'export\s+(\w+)\s*\(', line)
        if match:
            export_name = match.group(1)
            print(f"✓ Export: {export_name}")
            
            if self.active_environment:
                env_data = self.environments[self.active_environment]
                if 'exported' not in env_data:
                    env_data['exported'] = {}
                env_data['exported'][export_name] = line
                
                # Handle special exports
                if export_name == 'byp':
                    self.handle_bypass_export(line)
                elif export_name == 'env':
                    self.handle_env_export(line)
    
    def handle_activate_keyword(self, line: str):
        """Handle activate= keyword for activation mode"""
        if '-mode SCRIPT' in line:
            print(f"✓ Activate mode: SCRIPT")
            if self.active_environment:
                self.environments[self.active_environment]['mode'] = 'SCRIPT'
    
    def handle_bypass_export(self, line: str):
        """Handle bypass export for command execution"""
        print(f"✓ Bypass export configured")
        
        # Extract file patterns
        if '.py' in line:
            print(f"  ↳ Python file execution enabled")
        if 'terminal' in line:
            print(f"  ↳ Terminal mode enabled")
        if '-cmdbin' in line:
            print(f"  ↳ Command binary mode enabled")
        if '-byp' in line:
            print(f"  ↳ Bypass flag set")
    
    def handle_env_export(self, line: str):
        """Handle environment export"""
        print(f"✓ Environment export configured")
        if self.active_environment:
            env_name = self.active_environment
            print(f"  ↳ Exporting environment: {env_name}")
    
    def handle_try_block(self, line: str):
        """Handle try blocks"""
        # Extract content in try(...)
        match = re.search(r'try\s*\((.+)\)', line, re.DOTALL)
        if match:
            try_content = match.group(1).strip()
            print(f"✓ Try block: {try_content[:50]}...")
            # Process the content inside try
            self.process_line(try_content)
    
    def handle_try_assignment(self, var_name: str, var_value: str):
        """Handle assignments with try() blocks"""
        match = re.search(r'try\s*\((.+)\)', var_value)
        if match:
            content = match.group(1).strip()
            self.variables[var_name] = content
            print(f"✓ Set {var_name} with try block: {content}")
    
    def handle_dirlist(self, line: str):
        """Handle dirlist with -glob syntax"""
        print(f"✓ Directory list operation")
        
        # Extract glob pattern
        if '-glob default' in line:
            print(f"  ↳ Using default glob pattern")
        
        # Extract output
        if '-out' in line:
            match = re.search(r'-out "([^"]+)"', line)
            if match:
                output = match.group(1)
                print(f"  ↳ Output to: {output}")
        
        # Handle inline import
        if '@DEVRC.IMPORT=' in line:
            print(f"  ↳ With import reference")
        
        # Set variable
        self.variables['dirlist'] = "./"
        
    def handle_currentdir(self, line: str):
        """Handle currentdir = dirlist './' this.dir"""
        print(f"✓ Current directory operation")
        
        if 'this.dir' in line:
            print(f"  ↳ Using this.dir reference")
        
        current_dir = self.variables.get('currentdir', os.getcwd())
        self.variables['currentdir'] = current_dir
        print(f"  ↳ Current dir: {current_dir}")
    
    def handle_subenv(self, line: str):
        """Handle subenv = env.category"""
        print(f"✓ Sub-environment configuration")
        
        if 'env.category' in line:
            if self.active_environment:
                env_data = self.environments[self.active_environment]
                env_data['subenv'] = {'category': 'default'}
                print(f"  ↳ Sub-environment category set")
    
    def handle_environment_category(self, line: str):
        """Handle prod/dev/debug environment categories"""
        # Parse: prod=drizzle[content+subenv=["debug","prod","dev"]]
        match = re.match(r'(\w+)=(\w+)\[(.+)\]', line)
        if match:
            category = match.group(1)
            env_name = match.group(2)
            content = match.group(3)
            
            print(f"✓ Environment category: {category}")
            print(f"  ↳ Environment: {env_name}")
            
            # Parse subenv array
            if 'subenv=' in content:
                subenv_match = re.search(r'subenv=\[([^\]]+)\]', content)
                if subenv_match:
                    subenvs = [s.strip('"') for s in subenv_match.group(1).split(',')]
                    print(f"  ↳ Sub-environments: {', '.join(subenvs)}")
                    
                    if self.active_environment:
                        env_data = self.environments[self.active_environment]
                        env_data['categories'] = subenvs
    
    def handle_linenum(self, line: str):
        """Handle linenum = this.lines.fetched (-out is numerics)"""
        print(f"✓ Line number operation")
        
        if 'this.lines.fetched' in line:
            print(f"  ↳ Fetching line numbers")
        
        if '-out is numerics' in line:
            print(f"  ↳ Output as numerics")
        
        self.variables['linenum'] = 0
    
    def handle_current_line(self, line: str):
        """Handle current line with -activeline"""
        print(f"✓ Current line operation")
        
        if '-linenum' in line:
            print(f"  ↳ Using line numbers")
        
        if '-getline' in line:
            print(f"  ↳ Getting line content")
        
        if '-activeline' in line:
            print(f"  ↳ Active line mode enabled")
        
        if 'currentdir' in line:
            print(f"  ↳ From current directory")
        
        if 'get content[null]' in line:
            print(f"  ↳ Getting null content")
    
    def handle_get_operation(self, line: str):
        """Handle get operations for fetching data"""
        print(f"✓ Get operation")
        
        # Handle table[content] access
        if 'table[content]' in line:
            print(f"  ↳ Accessing table content")
        
        # Handle file operations
        if 'file' in line and 'file_ext' in line:
            print(f"  ↳ File retrieval operation")
        
        # Handle content[null]
        if 'content[null]' in line:
            print(f"  ↳ Accessing null content")
        
        # Handle glob patterns
        if '-glob' in line:
            print(f"  ↳ Using glob pattern")
    
    def handle_in_operation(self, line: str):
        """Handle in operations for context/scope"""
        print(f"✓ In operation")
        
        # Handle env[activate] access
        if 'env[activate]' in line:
            print(f"  ↳ Environment activation context")
            if self.active_environment:
                print(f"  ↳ Active environment: {self.active_environment}")
        
        # Handle env[content]
        if 'env[content]' in line:
            print(f"  ↳ Environment content context")
        
        # Handle file is STR
        if 'file is STR' in line:
            print(f"  ↳ File as string context")
        
        # Handle -glob default
        if '-glob default' in line:
            print(f"  ↳ Default glob pattern")
        
        # Handle this.* references
        if 'this.' in line:
            this_ref = re.search(r'this\.(\w+)', line)
            if this_ref:
                print(f"  ↳ This reference: {this_ref.group(1)}")
    
    def handle_devrc_command(self, tokens: List[str]):
        """Handle .devrc specific commands"""
        i = 0
        while i < len(tokens):
            token = tokens[i]
            
            if token == '-out' and i + 1 < len(tokens):
                self.output_to_file(tokens[i + 1])
                i += 2
            
            elif token == '-crfolder' and i + 1 < len(tokens):
                self.create_folder(tokens[i + 1])
                i += 2
            
            elif token == '-pop':
                if i + 1 < len(tokens):
                    print(f"✓ Pop operation: {tokens[i + 1]}")
                i += 2
            
            elif token == '-plugin':
                print("✓ Plugin mode enabled")
                i += 1
            
            elif token == '-config':
                print("✓ Config mode enabled")
                i += 1
            
            elif token == '-c':
                print("✓ Compile mode enabled")
                i += 1
            
            elif token == '-timed':
                print("✓ Timed operation enabled")
                i += 1
            
            elif token == '-mode' and i + 1 < len(tokens):
                print(f"✓ Mode set to: {tokens[i + 1]}")
                i += 2
            
            elif token == '-force':
                print("✓ Force mode enabled")
                i += 1
            
            elif token == '-a':
                print("✓ Append operation")
                i += 1
            
            elif token == '-locate' and i + 1 < len(tokens):
                print(f"✓ Locate: {tokens[i + 1]}")
                i += 2
            
            elif token == '-to':
                print("✓ Transform operation")
                i += 1
            
            elif token == '-ext' and i + 1 < len(tokens):
                print(f"✓ Extension: {tokens[i + 1]}")
                i += 2
            
            elif token == '-cmdbin':
                print("✓ Command binary mode")
                i += 1
            
            elif token == '-cmdline':
                print("✓ Command line mode")
                i += 1
            
            elif token == '-rline' and i + 1 < len(tokens):
                print(f"✓ Run line: {tokens[i + 1]}")
                i += 2
            
            elif token == '-r' and i + 1 < len(tokens):
                print(f"✓ Run mode: {tokens[i + 1]}")
                i += 2
            
            elif token == '-byp':
                print("✓ Bypass mode enabled")
                i += 1
            
            elif token == '-h' and i + 1 < len(tokens):
                print(f"✓ Handle pattern: {tokens[i + 1]}")
                i += 2
            
            elif token == '-ch':
                print("✓ Chain operation")
                i += 1
            
            elif token == '-numline':
                print("✓ Number line mode")
                i += 1
            
            elif token == '-ff':
                print("✓ Fast forward mode")
                i += 1
            
            else:
                i += 1
    
    def handle_if_statement(self, line: str):
        """Handle if statements"""
        # Extract condition
        match = re.search(r'if \((.*?)\) is (.*?)(?:\s+do\s+|\s+|$)', line)
        if match:
            var_name = match.group(1).strip()
            expected = match.group(2).strip()
            
            var_value = self.variables.get(var_name, False)
            expected_value = self.evaluate_expression(expected)
            
            if var_value == expected_value:
                # Execute the rest of the line
                rest = line[match.end():].strip()
                if rest:
                    print(f"✓ Condition met: {var_name} is {expected_value}")
                    self.process_line(rest)
            else:
                print(f"✗ Condition not met: {var_name} is not {expected_value}")
    
    def handle_for_loop(self, line: str):
        """Handle for loops"""
        match = re.search(r'for \((.*?)\)', line)
        if match:
            var_name = match.group(1).strip()
            print(f"✓ For loop over: {var_name}")
            # Execute the rest of the line
            rest = line[match.end():].strip()
            if rest:
                self.process_line(rest)
    
    def handle_do_statement(self, tokens: List[str]):
        """Handle do statements"""
        print(f"✓ Do statement: {' '.join(tokens)}")
        self.handle_devrc_command(tokens)
    
    def execute_section(self, section_name: str):
        """Execute a specific section"""
        if section_name not in self.sections:
            print(f"✗ Section not found: {section_name}")
            return
        
        section_type = self.section_types.get(section_name, "untyped")
        print(f"\n=== Executing section: {section_name} @[{section_type}] ===")
        for line in self.sections[section_name]:
            self.process_line(line)
    
    def execute_all(self):
        """Execute all sections in order"""
        for section_name, lines in self.sections.items():
            self.execute_section(section_name)
    
    def run(self, filepath: str, sections: Optional[List[str]] = None):
        """Run the DevRC interpreter"""
        print(f"DevRC Interpreter - Loading {filepath}")
        print(f"Root directory: {self.root_dir}")
        
        self.sections = self.parse_file(filepath)
        
        print(f"\n✓ Total sections loaded: {len(self.sections)}")
        print(f"✓ Total imports processed: {len(self.imported_files)}")
        if self.active_environment:
            print(f"✓ Active environment: {self.active_environment}")
        
        if sections:
            for section in sections:
                self.execute_section(section)
        else:
            self.execute_all()
        
        print("\n=== Execution complete ===")
        if self.imported_files:
            print(f"Imported files:")
            for imp in self.imported_files:
                print(f"  - {imp}")
        
        if self.environments:
            print(f"\nEnvironments:")
            for env_name, env_info in self.environments.items():
                active = " (active)" if env_name == self.active_environment else ""
                print(f"  - {env_name}{active}")
                print(f"    Path: {env_info['path']}")
        
        # Return to root directory after execution
        if self.active_environment:
            os.chdir(self.root_dir)
            print(f"\n✓ Returned to root directory: {self.root_dir}")


def main():
    import argparse
    
    parser = argparse.ArgumentParser(description='DevRC DSL Interpreter')
    parser.add_argument('file', help='.devrc file to execute')
    parser.add_argument('--section', '-s', action='append', 
                       help='Specific section(s) to execute')
    parser.add_argument('--dry-run', '-d', action='store_true',
                       help='Parse without executing')
    parser.add_argument('--root', '-r', 
                       help='Set root directory for environments (default: current directory)')
    parser.add_argument('--list-envs', action='store_true',
                       help='List all available environments')
    
    args = parser.parse_args()
    
    interpreter = DevRCInterpreter()
    
    # Set custom root if provided
    if args.root:
        interpreter.root_dir = os.path.abspath(args.root)
        print(f"Root directory set to: {interpreter.root_dir}")
    
    if args.dry_run:
        sections = interpreter.parse_file(args.file)
        print("Parsed sections:")
        for name, lines in sections.items():
            section_type = interpreter.section_types.get(name, "untyped")
            print(f"\n@[{section_type}]")
            print(f"[{name}]")
            for line in lines:
                print(f"  {line}")
        
        if interpreter.environments:
            print("\n\nEnvironments found:")
            for env_name, env_info in interpreter.environments.items():
                print(f"  - {env_name}: {env_info['path']}")
    
    elif args.list_envs:
        # Scan for environment directories in root
        print(f"Scanning for environments in: {interpreter.root_dir}")
        if os.path.exists(interpreter.root_dir):
            for item in os.listdir(interpreter.root_dir):
                item_path = os.path.join(interpreter.root_dir, item)
                if os.path.isdir(item_path):
                    print(f"  - {item}")
    
    else:
        interpreter.run(args.file, args.section)


if __name__ == '__main__':
    main()            elif token == '-cmdline':
                print("✓ Command line mode")
                i += 1
            
            elif token == '-rline' and i + 1 < len(tokens):
                print(f"✓ Run line: {tokens[i + 1]}")
                i += 2
            
            elif token == '-r' and i + 1 < len(tokens):
                print(f"✓ Run mode: {tokens[i + 1]}")
                i += 2
            
            elif token == '-byp':
                print("✓ Bypass mode enabled")
                i += 1
            
            elif token == '-h' and i + 1 < len(tokens):
                print(f"✓ Handle pattern: {tokens[i + 1]}")
                i += 2
            
            elif token == '-ch':
                print("✓ Chain operation")
                i += 1
            
            elif token == '-numline':
                print("✓ Number line mode")
                i += 1
            
            elif token == '-ff':
                print("✓ Fast forward mode")
                i += 1
            
            elif token == '-glob' and i + 1 < len(tokens):
                print(f"✓ Glob pattern: {tokens[i + 1]}")
                i += 2
            
            elif token == '-set':
                print("✓ Set operation")
                i += 1
            
            elif token == '-getline':
                print("✓ Get line operation")
                i += 1
            
            elif token == '-linenum':
                print("✓ Line number operation")
                i += 1
            
            elif token == '-activeline':
                print("✓ Active line mode")
                i += 1
            
            elif token == '-enable':
                print("✓ Enable flag")
                i += 1
            
            elif token == '-commitline':
                print("✓ Commit line operation")
                i += 1
            
            else:
                i += 1#!/usr/bin/env python3
"""
DevRC DSL Interpreter
Parses and executes .devrc configuration files
"""

import re
import os
import json
import subprocess
from pathlib import Path
from typing import Dict, List, Any, Optional


class DevRCInterpreter:
    def __init__(self):
        self.variables = {}
        self.sections = {}
        self.section_types = {}
        self.current_section = None
        self.imported_files = set()
        self.import_stack = []
        self.environments = {}
        self.active_environment = None
        self.root_dir = os.getcwd()
        
    def parse_file(self, filepath: str) -> Dict[str, List[str]]:
        """Parse a .devrc file into sections"""
        # Prevent circular imports
        abs_path = os.path.abspath(filepath)
        if abs_path in self.import_stack:
            print(f"✗ Circular import detected: {filepath}")
            return {}
        
        self.import_stack.append(abs_path)
        
        with open(filepath, 'r') as f:
            content = f.read()
        
        sections = {}
        current_section = None
        current_type = None
        
        for line in content.split('\n'):
            original_line = line
            
            # Check for environment activation BEFORE removing comments
            if line.strip().startswith('#[') and ']/ACTIVATE' in line:
                self.handle_environment_activation(line.strip())
                continue
            
            # Check for inline imports with @DEVRC.IMPORT=
            if '@DEVRC.IMPORT=' in line:
                self.handle_inline_import(line)
                # Don't skip the line, let it be processed
            
            # Remove comments (but not environment markers)
            if '#' in line and not line.strip().startswith('#['):
                line = line.split('#')[0]
            
            line = line.strip()
            if not line:
                continue
            
            # Check for imports
            if line.startswith('@DEVRC.IMPORT.'):
                self.handle_import(line, os.path.dirname(filepath))
                continue
            
            # Check for type annotations
            if line.startswith('@[') and line.endswith(']'):
                current_type = line[2:-1]
                print(f"✓ Type annotation found: {current_type}")
                continue
                
            # Check for section headers
            if line.startswith('[') and line.endswith(']'):
                current_section = line[1:-1]
                sections[current_section] = []
                if current_type:
                    self.section_types[current_section] = current_type
                    current_type = None
            elif current_section:
                sections[current_section].append(line)
        
        self.import_stack.pop()
        return sections
    
    def tokenize(self, line: str) -> List[str]:
        """Tokenize a line into components"""
        # Handle quoted strings
        tokens = []
        current = []
        in_quotes = False
        
        for char in line:
            if char == '"':
                in_quotes = not in_quotes
                current.append(char)
            elif char in [' ', '\t'] and not in_quotes:
                if current:
                    tokens.append(''.join(current))
                    current = []
            else:
                current.append(char)
        
        if current:
            tokens.append(''.join(current))
        
        return tokens
    
    def parse_assignment(self, line: str) -> Optional[tuple]:
        """Parse variable assignment"""
        if '=' in line:
            parts = line.split('=', 1)
            var_name = parts[0].strip()
            var_value = parts[1].strip()
            return (var_name, var_value)
        return None
    
    def evaluate_expression(self, expr: str) -> Any:
        """Evaluate an expression"""
        expr = expr.strip()
        
        # Remove quotes
        if expr.startswith('"') and expr.endswith('"'):
            return expr[1:-1]
        
        # Check if it's a variable reference
        if expr in self.variables:
            return self.variables[expr]
        
        # Check for boolean literals
        if expr.lower() in ['true', '-true']:
            return True
        if expr.lower() in ['false', '-false']:
            return False
        
        # Check for null
        if expr.lower() == 'null':
            return None
        
        return expr
    
    def handle_import(self, line: str, base_path: str):
        """Handle @DEVRC.IMPORT.[variablename] statements"""
        # Parse import statement: @DEVRC.IMPORT.[variablename] or @DEVRC.IMPORT.[variablename]="path"
        match = re.match(r'@DEVRC\.IMPORT\.(\w+)(?:="?([^"]+)"?)?', line)
        if not match:
            print(f"✗ Invalid import syntax: {line}")
            return
        
        var_name = match.group(1)
        import_path = match.group(2)
        
        # If no path specified, check if variable exists
        if not import_path:
            if var_name not in self.variables:
                print(f"✗ Import failed: variable '{var_name}' not defined")
                return
            import_path = self.variables[var_name]
        
        # Resolve relative paths
        if not os.path.isabs(import_path):
            import_path = os.path.join(base_path, import_path)
        
        # Check if already imported
        abs_import_path = os.path.abspath(import_path)
        if abs_import_path in self.imported_files:
            print(f"✓ Already imported: {import_path}")
            return
        
        # Check if file exists
        if not os.path.exists(import_path):
            print(f"✗ Import file not found: {import_path}")
            return
        
        print(f"✓ Importing from: {import_path}")
        self.imported_files.add(abs_import_path)
        
        # Parse and merge the imported file
        imported_sections = self.parse_file(import_path)
        for section_name, lines in imported_sections.items():
            if section_name in self.sections:
                # Merge with existing section
                print(f"  ↳ Merging section: [{section_name}]")
                self.sections[section_name].extend(lines)
            else:
                # Add new section
                print(f"  ↳ Adding section: [{section_name}]")
                self.sections[section_name] = lines
                # Copy type if exists
                if section_name in self.section_types:
                    self.section_types[section_name] = self.section_types[section_name]
    
    def handle_inline_import(self, line: str):
        """Handle inline @DEVRC.IMPORT= statements within expressions"""
        # Parse: @DEVRC.IMPORT="./file.devrc" or @DEVRC.IMPORT="dirlist"
        match = re.search(r'@DEVRC\.IMPORT="([^"]+)"', line)
        if match:
            import_ref = match.group(1)
            print(f"✓ Inline import reference: {import_ref}")
            
            # Store as variable for later use
            if 'is STR' in line:
                var_match = re.search(r'is STR "([^"]+)"', line)
                if var_match:
                    var_name = var_match.group(1)
                    self.variables[var_name] = import_ref
                    print(f"  ↳ Stored as: {var_name}")
    
    def handle_environment_activation(self, line: str):
        """Handle #[environmentname]/ACTIVATE directives"""
        # Parse: #[environmentname]/ACTIVATE
        match = re.match(r'#\[([^\]]+)\]/ACTIVATE', line)
        if not match:
            print(f"✗ Invalid environment activation syntax: {line}")
            return
        
        env_name = match.group(1)
        
        # Create environment directory path from root
        env_path = os.path.join(self.root_dir, env_name)
        
        # Create environment if it doesn't exist
        if not os.path.exists(env_path):
            try:
                os.makedirs(env_path, exist_ok=True)
                print(f"✓ Created environment directory: {env_path}")
            except Exception as e:
                print(f"✗ Failed to create environment directory: {e}")
                return
        
        # Activate the environment
        self.active_environment = env_name
        self.environments[env_name] = {
            'path': env_path,
            'root': self.root_dir,
            'activated_at': os.getcwd(),
            'mode': 'SCRIPT',
            'exported': {},
            'subenv': {},
            'content': {}
        }
        
        # Set environment variable
        self.variables['env'] = env_name
        self.variables['activate'] = f"{env_name}/ACTIVATE"
        self.variables['currentdir'] = env_path
        
        # Change to environment directory
        try:
            os.chdir(env_path)
            print(f"✓ Environment activated: {env_name}")
            print(f"  ↳ Working directory: {env_path}")
            print(f"  ↳ Root directory: {self.root_dir}")
            print(f"  ↳ Mode: SCRIPT")
        except Exception as e:
            print(f"✗ Failed to change to environment directory: {e}")
    
    def create_folder(self, path: str):
        """Create a folder if it doesn't exist"""
        path = path.strip('"').replace('*', '')
        try:
            Path(path).mkdir(parents=True, exist_ok=True)
            print(f"✓ Created folder: {path}")
        except Exception as e:
            print(f"✗ Error creating folder {path}: {e}")
    
    def output_to_file(self, path: str, content: Any = None):
        """Handle output to file"""
        path = path.strip('"').replace('*', '')
        try:
            if '*' in path or path.endswith('/'):
                # Directory output
                Path(path).mkdir(parents=True, exist_ok=True)
                print(f"✓ Prepared output directory: {path}")
            else:
                # File output
                Path(path).parent.mkdir(parents=True, exist_ok=True)
                if content:
                    with open(path, 'w') as f:
                        if isinstance(content, dict):
                            json.dump(content, f, indent=2)
                        else:
                            f.write(str(content))
                print(f"✓ Output to: {path}")
        except Exception as e:
            print(f"✗ Error outputting to {path}: {e}")
    
    def execute_command(self, command: List[str]):
        """Execute a system command"""
        try:
            result = subprocess.run(command, capture_output=True, text=True)
            print(f"✓ Executed: {' '.join(command)}")
            return result.stdout
        except Exception as e:
            print(f"✗ Error executing command: {e}")
            return None
    
    def process_line(self, line: str):
        """Process a single line of DevRC code"""
        tokens = self.tokenize(line)
        if not tokens:
            return
        
        # Handle assignments with special syntax
        assignment = self.parse_assignment(line)
        if assignment:
            var_name, var_value = assignment
            
            # Handle special assignments like poot={}
            if var_value.strip() in ['{}', '[]']:
                self.variables[var_name] = {}
                print(f"✓ Initialized {var_name} as empty container")
                return
            
            # Handle complex assignments with try()
            if 'try (' in var_value:
                self.handle_try_assignment(var_name, var_value)
                return
            
            self.variables[var_name] = self.evaluate_expression(var_value)
            print(f"✓ Set {var_name} = {self.variables[var_name]}")
            return
        
        # Handle dirlist with -glob
        if tokens[0] == 'dirlist' or 'dirlist=' in line:
            self.handle_dirlist(line)
            return
        
        # Handle currentdir
        if tokens[0] == 'currentdir' or 'currentdir=' in line:
            self.handle_currentdir(line)
            return
        
        # Handle subenv
        if tokens[0] == 'subenv' or 'subenv=' in line:
            self.handle_subenv(line)
            return
        
        # Handle prod/dev/debug environments
        if tokens[0] in ['prod', 'dev', 'debug'] and '=' in line:
            self.handle_environment_category(line)
            return
        
        # Handle linenum
        if tokens[0] == 'linenum' or 'linenum=' in line:
            self.handle_linenum(line)
            return
        
        # Handle current with activeline
        if tokens[0] == 'current' or 'current=' in line:
            self.handle_current_line(line)
            return
        
        # Handle function definitions
        if tokens[0] == 'function':
            self.handle_function_definition(line)
            return
        
        # Handle return statements
        if tokens[0] == 'return':
            self.handle_return_statement(line)
            return
        
        # Handle export statements
        if tokens[0] == 'export':
            self.handle_export_statement(line)
            return
        
        # Handle activate keyword
        if tokens[0] == 'activate':
            self.handle_activate_keyword(line)
            return
        
        # Handle .devrc commands
        if tokens[0] == '.devrc':
            self.handle_devrc_command(tokens[1:])
        
        # Handle if statements
        elif tokens[0] == 'if':
            self.handle_if_statement(line)
        
        # Handle for loops
        elif tokens[0] == 'for':
            self.handle_for_loop(line)
        
        # Handle do statements
        elif tokens[0] == 'do':
            self.handle_do_statement(tokens[1:])
        
        # Handle out command
        elif tokens[0] == 'out':
            if len(tokens) > 1:
                self.output_to_file(tokens[1])
        
        # Handle get operations
        elif tokens[0] == 'get':
            self.handle_get_operation(line)
        
        # Handle in operations  
        elif tokens[0] == 'in':
            self.handle_in_operation(line)
        
        # Handle try blocks
        elif tokens[0] == 'try':
            self.handle_try_block(line)
    
    def handle_function_definition(self, line: str):
        """Handle function definitions"""
        match = re.search(r'function\s+(\w+)\s*\(', line)
        if match:
            func_name = match.group(1)
            print(f"✓ Function defined: {func_name}")
            self.variables[func_name] = "function"
        else:
            # Anonymous function or function call syntax
            print(f"✓ Function block defined")
    
    def handle_return_statement(self, line: str):
        """Handle return statements"""
        # Extract return value
        match = re.search(r'return\s+(.+)', line)
        if match:
            return_val = match.group(1).strip()
            print(f"✓ Return: {return_val}")
            if self.active_environment:
                self.environments[self.active_environment]['return_value'] = return_val
    
    def handle_export_statement(self, line: str):
        """Handle export statements for environment variables"""
        # Parse: export name ( ... )
        match = re.search(r'export\s+(\w+)\s*\(', line)
        if match:
            export_name = match.group(1)
            print(f"✓ Export: {export_name}")
            
            if self.active_environment:
                env_data = self.environments[self.active_environment]
                if 'exported' not in env_data:
                    env_data['exported'] = {}
                env_data['exported'][export_name] = line
                
                # Handle special exports
                if export_name == 'byp':
                    self.handle_bypass_export(line)
                elif export_name == 'env':
                    self.handle_env_export(line)
    
    def handle_activate_keyword(self, line: str):
        """Handle activate= keyword for activation mode"""
        if '-mode SCRIPT' in line:
            print(f"✓ Activate mode: SCRIPT")
            if self.active_environment:
                self.environments[self.active_environment]['mode'] = 'SCRIPT'
    
    def handle_bypass_export(self, line: str):
        """Handle bypass export for command execution"""
        print(f"✓ Bypass export configured")
        
        # Extract file patterns
        if '.py' in line:
            print(f"  ↳ Python file execution enabled")
        if 'terminal' in line:
            print(f"  ↳ Terminal mode enabled")
        if '-cmdbin' in line:
            print(f"  ↳ Command binary mode enabled")
        if '-byp' in line:
            print(f"  ↳ Bypass flag set")
    
    def handle_env_export(self, line: str):
        """Handle environment export"""
        print(f"✓ Environment export configured")
        if self.active_environment:
            env_name = self.active_environment
            print(f"  ↳ Exporting environment: {env_name}")
    
    def handle_try_block(self, line: str):
        """Handle try blocks"""
        # Extract content in try(...)
        match = re.search(r'try\s*\((.+)\)', line, re.DOTALL)
        if match:
            try_content = match.group(1).strip()
            print(f"✓ Try block: {try_content[:50]}...")
            # Process the content inside try
            self.process_line(try_content)
    
    def handle_try_assignment(self, var_name: str, var_value: str):
        """Handle assignments with try() blocks"""
        match = re.search(r'try\s*\((.+)\)', var_value)
        if match:
            content = match.group(1).strip()
            self.variables[var_name] = content
            print(f"✓ Set {var_name} with try block: {content}")
    
    def handle_dirlist(self, line: str):
        """Handle dirlist with -glob syntax"""
        print(f"✓ Directory list operation")
        
        # Extract glob pattern
        if '-glob default' in line:
            print(f"  ↳ Using default glob pattern")
        
        # Extract output
        if '-out' in line:
            match = re.search(r'-out "([^"]+)"', line)
            if match:
                output = match.group(1)
                print(f"  ↳ Output to: {output}")
        
        # Handle inline import
        if '@DEVRC.IMPORT=' in line:
            print(f"  ↳ With import reference")
        
        # Set variable
        self.variables['dirlist'] = "./"
        
    def handle_currentdir(self, line: str):
        """Handle currentdir = dirlist './' this.dir"""
        print(f"✓ Current directory operation")
        
        if 'this.dir' in line:
            print(f"  ↳ Using this.dir reference")
        
        current_dir = self.variables.get('currentdir', os.getcwd())
        self.variables['currentdir'] = current_dir
        print(f"  ↳ Current dir: {current_dir}")
    
    def handle_subenv(self, line: str):
        """Handle subenv = env.category"""
        print(f"✓ Sub-environment configuration")
        
        if 'env.category' in line:
            if self.active_environment:
                env_data = self.environments[self.active_environment]
                env_data['subenv'] = {'category': 'default'}
                print(f"  ↳ Sub-environment category set")
    
    def handle_environment_category(self, line: str):
        """Handle prod/dev/debug environment categories"""
        # Parse: prod=drizzle[content+subenv=["debug","prod","dev"]]
        match = re.match(r'(\w+)=(\w+)\[(.+)\]', line)
        if match:
            category = match.group(1)
            env_name = match.group(2)
            content = match.group(3)
            
            print(f"✓ Environment category: {category}")
            print(f"  ↳ Environment: {env_name}")
            
            # Parse subenv array
            if 'subenv=' in content:
                subenv_match = re.search(r'subenv=\[([^\]]+)\]', content)
                if subenv_match:
                    subenvs = [s.strip('"') for s in subenv_match.group(1).split(',')]
                    print(f"  ↳ Sub-environments: {', '.join(subenvs)}")
                    
                    if self.active_environment:
                        env_data = self.environments[self.active_environment]
                        env_data['categories'] = subenvs
    
    def handle_linenum(self, line: str):
        """Handle linenum = this.lines.fetched (-out is numerics)"""
        print(f"✓ Line number operation")
        
        if 'this.lines.fetched' in line:
            print(f"  ↳ Fetching line numbers")
        
        if '-out is numerics' in line:
            print(f"  ↳ Output as numerics")
        
        self.variables['linenum'] = 0
    
    def handle_current_line(self, line: str):
        """Handle current line with -activeline"""
        print(f"✓ Current line operation")
        
        if '-linenum' in line:
            print(f"  ↳ Using line numbers")
        
        if '-getline' in line:
            print(f"  ↳ Getting line content")
        
        if '-activeline' in line:
            print(f"  ↳ Active line mode enabled")
        
        if 'currentdir' in line:
            print(f"  ↳ From current directory")
        
        if 'get content[null]' in line:
            print(f"  ↳ Getting null content")
    
    def handle_get_operation(self, line: str):
        """Handle get operations for fetching data"""
        print(f"✓ Get operation")
        
        # Handle table[content] access
        if 'table[content]' in line:
            print(f"  ↳ Accessing table content")
        
        # Handle file operations
        if 'file' in line and 'file_ext' in line:
            print(f"  ↳ File retrieval operation")
        
        # Handle content[null]
        if 'content[null]' in line:
            print(f"  ↳ Accessing null content")
        
        # Handle glob patterns
        if '-glob' in line:
            print(f"  ↳ Using glob pattern")
    
    def handle_in_operation(self, line: str):
        """Handle in operations for context/scope"""
        print(f"✓ In operation")
        
        # Handle env[activate] access
        if 'env[activate]' in line:
            print(f"  ↳ Environment activation context")
            if self.active_environment:
                print(f"  ↳ Active environment: {self.active_environment}")
        
        # Handle env[content]
        if 'env[content]' in line:
            print(f"  ↳ Environment content context")
        
        # Handle file is STR
        if 'file is STR' in line:
            print(f"  ↳ File as string context")
        
        # Handle -glob default
        if '-glob default' in line:
            print(f"  ↳ Default glob pattern")
        
        # Handle this.* references
        if 'this.' in line:
            this_ref = re.search(r'this\.(\w+)', line)
            if this_ref:
                print(f"  ↳ This reference: {this_ref.group(1)}")
    
    def handle_devrc_command(self, tokens: List[str]):
        """Handle .devrc specific commands"""
        i = 0
        while i < len(tokens):
            token = tokens[i]
            
            if token == '-out' and i + 1 < len(tokens):
                self.output_to_file(tokens[i + 1])
                i += 2
            
            elif token == '-crfolder' and i + 1 < len(tokens):
                self.create_folder(tokens[i + 1])
                i += 2
            
            elif token == '-pop':
                if i + 1 < len(tokens):
                    print(f"✓ Pop operation: {tokens[i + 1]}")
                i += 2
            
            elif token == '-plugin':
                print("✓ Plugin mode enabled")
                i += 1
            
            elif token == '-config':
                print("✓ Config mode enabled")
                i += 1
            
            elif token == '-c':
                print("✓ Compile mode enabled")
                i += 1
            
            elif token == '-timed':
                print("✓ Timed operation enabled")
                i += 1
            
            elif token == '-mode' and i + 1 < len(tokens):
                print(f"✓ Mode set to: {tokens[i + 1]}")
                i += 2
            
            elif token == '-force':
                print("✓ Force mode enabled")
                i += 1
            
            elif token == '-a':
                print("✓ Append operation")
                i += 1
            
            elif token == '-locate' and i + 1 < len(tokens):
                print(f"✓ Locate: {tokens[i + 1]}")
                i += 2
            
            elif token == '-to':
                print("✓ Transform operation")
                i += 1
            
            elif token == '-ext' and i + 1 < len(tokens):
                print(f"✓ Extension: {tokens[i + 1]}")
                i += 2
            
            elif token == '-cmdbin':
                print("✓ Command binary mode")
                i += 1
            
            elif token == '-cmdline':
                print("✓ Command line mode")
                i += 1
            
            elif token == '-rline' and i + 1 < len(tokens):
                print(f"✓ Run line: {tokens[i + 1]}")
                i += 2
            
            elif token == '-r' and i + 1 < len(tokens):
                print(f"✓ Run mode: {tokens[i + 1]}")
                i += 2
            
            elif token == '-byp':
                print("✓ Bypass mode enabled")
                i += 1
            
            elif token == '-h' and i + 1 < len(tokens):
                print(f"✓ Handle pattern: {tokens[i + 1]}")
                i += 2
            
            elif token == '-ch':
                print("✓ Chain operation")
                i += 1
            
            elif token == '-numline':
                print("✓ Number line mode")
                i += 1
            
            elif token == '-ff':
                print("✓ Fast forward mode")
                i += 1
            
            else:
                i += 1
    
    def handle_if_statement(self, line: str):
        """Handle if statements"""
        # Extract condition
        match = re.search(r'if \((.*?)\) is (.*?)(?:\s+do\s+|\s+|$)', line)
        if match:
            var_name = match.group(1).strip()
            expected = match.group(2).strip()
            
            var_value = self.variables.get(var_name, False)
            expected_value = self.evaluate_expression(expected)
            
            if var_value == expected_value:
                # Execute the rest of the line
                rest = line[match.end():].strip()
                if rest:
                    print(f"✓ Condition met: {var_name} is {expected_value}")
                    self.process_line(rest)
            else:
                print(f"✗ Condition not met: {var_name} is not {expected_value}")
    
    def handle_for_loop(self, line: str):
        """Handle for loops"""
        match = re.search(r'for \((.*?)\)', line)
        if match:
            var_name = match.group(1).strip()
            print(f"✓ For loop over: {var_name}")
            # Execute the rest of the line
            rest = line[match.end():].strip()
            if rest:
                self.process_line(rest)
    
    def handle_do_statement(self, tokens: List[str]):
        """Handle do statements"""
        print(f"✓ Do statement: {' '.join(tokens)}")
        self.handle_devrc_command(tokens)
    
    def execute_section(self, section_name: str):
        """Execute a specific section"""
        if section_name not in self.sections:
            print(f"✗ Section not found: {section_name}")
            return
        
        section_type = self.section_types.get(section_name, "untyped")
        print(f"\n=== Executing section: {section_name} @[{section_type}] ===")
        for line in self.sections[section_name]:
            self.process_line(line)
    
    def execute_all(self):
        """Execute all sections in order"""
        for section_name, lines in self.sections.items():
            self.execute_section(section_name)
    
    def run(self, filepath: str, sections: Optional[List[str]] = None):
        """Run the DevRC interpreter"""
        print(f"DevRC Interpreter - Loading {filepath}")
        print(f"Root directory: {self.root_dir}")
        
        self.sections = self.parse_file(filepath)
        
        print(f"\n✓ Total sections loaded: {len(self.sections)}")
        print(f"✓ Total imports processed: {len(self.imported_files)}")
        if self.active_environment:
            print(f"✓ Active environment: {self.active_environment}")
        
        if sections:
            for section in sections:
                self.execute_section(section)
        else:
            self.execute_all()
        
        print("\n=== Execution complete ===")
        if self.imported_files:
            print(f"Imported files:")
            for imp in self.imported_files:
                print(f"  - {imp}")
        
        if self.environments:
            print(f"\nEnvironments:")
            for env_name, env_info in self.environments.items():
                active = " (active)" if env_name == self.active_environment else ""
                print(f"  - {env_name}{active}")
                print(f"    Path: {env_info['path']}")
        
        # Return to root directory after execution
        if self.active_environment:
            os.chdir(self.root_dir)
            print(f"\n✓ Returned to root directory: {self.root_dir}")


def main():
    import argparse
    
    parser = argparse.ArgumentParser(description='DevRC DSL Interpreter')
    parser.add_argument('file', help='.devrc file to execute')
    parser.add_argument('--section', '-s', action='append', 
                       help='Specific section(s) to execute')
    parser.add_argument('--dry-run', '-d', action='store_true',
                       help='Parse without executing')
    parser.add_argument('--root', '-r', 
                       help='Set root directory for environments (default: current directory)')
    parser.add_argument('--list-envs', action='store_true',
                       help='List all available environments')
    
    args = parser.parse_args()
    
    interpreter = DevRCInterpreter()
    
    # Set custom root if provided
    if args.root:
        interpreter.root_dir = os.path.abspath(args.root)
        print(f"Root directory set to: {interpreter.root_dir}")
    
    if args.dry_run:
        sections = interpreter.parse_file(args.file)
        print("Parsed sections:")
        for name, lines in sections.items():
            section_type = interpreter.section_types.get(name, "untyped")
            print(f"\n@[{section_type}]")
            print(f"[{name}]")
            for line in lines:
                print(f"  {line}")
        
        if interpreter.environments:
            print("\n\nEnvironments found:")
            for env_name, env_info in interpreter.environments.items():
                print(f"  - {env_name}: {env_info['path']}")
    
    elif args.list_envs:
        # Scan for environment directories in root
        print(f"Scanning for environments in: {interpreter.root_dir}")
        if os.path.exists(interpreter.root_dir):
            for item in os.listdir(interpreter.root_dir):
                item_path = os.path.join(interpreter.root_dir, item)
                if os.path.isdir(item_path):
                    print(f"  - {item}")
    
    else:
        interpreter.run(args.file, args.section)


if __name__ == '__main__':
    main()
