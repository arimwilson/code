#!/usr/bin/env python3
import os
import argparse
import re
import platform
import shutil
import subprocess

def process_directory(base_dir, current_dir, include_ext, include_files, exclude_substrings, files_found):
    """
    Recursively process the directory tree.
    For each directory, first process files (alphabetically) then process subdirectories (alphabetically).
    """
    output_text = ""
    
    # Get sorted list of files and directories in the current directory.
    entries = sorted(os.listdir(current_dir))
    files = [entry for entry in entries if os.path.isfile(os.path.join(current_dir, entry))]
    dirs  = [entry for entry in entries if os.path.isdir(os.path.join(current_dir, entry))]
    
    # Process files in alphabetical order.
    for filename in files:
        full_path = os.path.join(current_dir, filename)
        rel_path = os.path.relpath(full_path, base_dir)
        
        # If exclude substrings are provided, skip files whose full path contains any of them.
        # Use substring matching ('in') instead of regex.
        should_exclude = False
        if exclude_substrings:
            if any(substring in full_path for substring in exclude_substrings):
                should_exclude = True

        if should_exclude:
            continue

        # Check if the file matches an accepted extension.
        matched = any(filename.endswith(ext) for ext in include_ext)
        # Or if the file exactly matches one of the specific filenames.
        if not matched and filename in include_files:
            matched = True

        if matched:
            # Read file content. Try utf-8 first; fallback to latin-1 if needed.
            try:
                with open(full_path, 'r', encoding='utf-8') as f:
                    content = f.read()
            except UnicodeDecodeError:
                with open(full_path, 'r', encoding='latin-1') as f:
                    content = f.read()
            
            # Append file path, file content, and a separator line.
            output_text += f"{rel_path}:\n{content}\n---------------------------\n"
            files_found.append(rel_path)
    
    # Process subdirectories in alphabetical order.
    for d in dirs:
        output_text += process_directory(base_dir, os.path.join(current_dir, d), include_ext, include_files, exclude_substrings, files_found)
    
    return output_text

def copy_to_clipboard(text):
    system_name = platform.system()
    if system_name == "Darwin":
        # Check if pbcopy is available.
        if shutil.which("pbcopy") is None:
            raise EnvironmentError("pbcopy command not found on macOS.")
        subprocess.run(["pbcopy"], input=text.encode("utf-8"), check=True)
    elif system_name == "Linux":
        # Check if xclip is available.
        if shutil.which("xclip") is None:
            raise EnvironmentError("xclip command not found on Linux.")
        subprocess.run(["xclip", "-selection", "clipboard"], input=text.encode("utf-8"), check=True)
    else:
        raise EnvironmentError("Unsupported operating system. This script supports only macOS and Linux.")

def main():
    parser = argparse.ArgumentParser(description="Concatenate source files for LLM input and copy to clipboard.")
    parser.add_argument('-d', '--directory', default='.', 
                        help='Top-level directory to search (default: current directory)')
    parser.add_argument('-e', '--extensions', default='.ts,.js,.html,.java',
                        help='Comma-separated list of accepted file extensions (default: .ts,.js,.html,.java)')
    parser.add_argument('-f', '--files', default='pom.xml,package.json',
                        help='Comma-separated list of accepted file names (default: pom.xml,package.json)')
    parser.add_argument('-x', '--exclude', default='node_modules',
                        help='Comma-separated list of substrings. Files/directories whose full path contains any of these substrings will be excluded (default: none)')
    args = parser.parse_args()

    base_dir = os.path.abspath(args.directory)
    # Split and strip the allowed file extensions and file names.
    include_ext = [ext.strip() for ext in args.extensions.split(',') if ext.strip()]
    include_files = [fname.strip() for fname in args.files.split(',') if fname.strip()]
    # Process exclude argument: split by comma, strip whitespace, filter empty strings
    exclude_substrings = []
    if args.exclude:
        exclude_substrings = [s.strip() for s in args.exclude.split(',') if s.strip()]

    # List to store the relative paths of the files that are included.
    files_found = []

    # Recursively process the directory structure.
    output_text = process_directory(base_dir, base_dir, include_ext, include_files, exclude_substrings, files_found)
    
    # Copy the concatenated text to the system clipboard using platform-specific commands.
    copy_to_clipboard(output_text)
    
    # Output the list of files that were copied.
    print("Files copied to clipboard:")
    for f in files_found:
        print(f)

if __name__ == '__main__':
    main()
