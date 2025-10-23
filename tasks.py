import re
import json


def parse_milestone(md_text):
    """
    Parses a Markdown milestone into a structured list of GitHub issues.
    
    The script:
    1. Splits the document by H2 headers (##) to isolate main issues.
    2. Parses metadata (Estimate, Labels, Priority, Objective) for each main issue.
    3. Parses the main issue's tasklist, converting top-level items into separate subtasks.
    4. Converts nested checklist items (indented with a dash) into body content 
       for their parent subtask (since GitHub Issues don't natively nest).
    
    The resulting structure facilitates a clean import into a platform like GitHub,
    where a main issue will track a set of smaller, atomic issues (subtasks).
    """
    
    # 1. Split the document by H2 header (##) to get sections/issues
    sections = re.split(r'\n##\s+', md_text)[1:] # Skip the initial milestone title
    
    iterations = []
    
    for i, section_text in enumerate(sections):
        lines = section_text.strip().split('\n')
        
        # Parse Main Issue Header and Title
        # Title is the first line, strip the number/dot/space (e.g., "1. ")
        match_title = re.match(r'(\d+\.\s)?(.+)', lines[0].strip())
        main_issue_title = match_title.group(2).strip() if match_title else lines[0].strip()
        iteration_name = f"Iteration {i + 1}: {main_issue_title}"
        
        # Parse Metadata (Estimate, Labels, Priority)
        metadata = {}
        content_lines = []
        in_metadata_block = True
        
        for line in lines[1:]:
            #line = line.strip()
            if not line:
                continue
            
            if in_metadata_block:
                # Regex to match key: value (e.g., "Labels: ops, logging")
                match_meta = re.match(r'(.+?):\s*(.+)', line)
                if match_meta:
                    key = match_meta.group(1).strip().lower().replace('-', '_')
                    value = match_meta.group(2).strip()
                    metadata[key] = value
                elif line.lower() == "sub-tasks":
                    # Sub-Tasks is a delimiter, the rest is issue body and tasks
                    in_metadata_block = False
                elif line.lower().startswith("objective:"):
                    # Handle objective as a special metadata item
                    metadata['objective'] = line.split(':', 1)[1].strip()
                else:
                    # Anything else not matching is the start of the body content
                    in_metadata_block = False
                    content_lines.append(line)
            else:
                content_lines.append(line)

        # 2. Parse Tasklist and create Subtasks
        current_parent_title = None
        current_subtask_body = []
        
        # Combine the rest of the lines into a single string for easier regex parsing of tasks
        content = "\n".join(content_lines)
        
        # Regex to find all top-level and nested checklist items
        # Group 1: indent (0 or 2 spaces)
        # Group 2: checkbox status ([ ] or [x])
        # Group 3: task text
        task_regex = re.compile(r'^(\s*)(-)\s+\[[ x]\]\s*(.+)', re.MULTILINE)
        taskstack = []
        tasks = []
        level = 0
        
        for line in content.split('\n'):
            match = task_regex.match(line)
            if not match:
                continue

            indent = match.group(1)
            task_text = match.group(3).strip()
            
            if len(indent) > level:
                level = len(indent)
                taskstack.append(tasks)
                tasks = []
            elif len(indent) < level:
                level = len(indent)
                prevtasks = taskstack.pop()
                prevtasks[len(prevtasks)-1]['tasks'] = tasks
                tasks = prevtasks

            tasks.append({
                "title": task_text,
                "labels": metadata.get('labels', '') # Inherit labels from parent
            })

        # 3. Create the Parent Epic Issue
        parent_issue = {
            "title": f"EPIC: {main_issue_title}",
            "body": f"**Objective:** {metadata.get('objective', 'N/A')}\n\n**Estimate:** {metadata.get('estimate', 'N/A')}\n**Priority:** {metadata.get('priority', 'N/A')}\n\n---",
            "iteration": iteration_name, # Parent also belongs to the iteration
            "labels": metadata.get('labels', '') + ", epic",
            "tasks": tasks # Add the subtasks for tracking
        }
        
        iterations.append(parent_issue)
    
    return iterations

# --- Execution ---

with open("MILESTONE.md", "r") as fd:
    markdown_milestone = fd.read()
    parsed_issues = parse_milestone(markdown_milestone)

"""
print("✅ Successfully parsed milestone into structured issues. ")
print("\n--- Summary of Issues to Create ---")

for issue in parsed_issues:
    print(f"\n--- PARENT ISSUE/EPIC ---")
    print(f"TITLE: {issue['title']}")
    print(f"ITERATION: {issue['iteration']}")
    print(f"LABELS: {issue['labels']}")
    print(f"TASK COUNT: {len(issue['tasks'])}")

    for task in issue['tasks']:
        print(f"  - TASK TITLE: {task['title']}")

print("\n--- Full JSON Output (Simplified) ---")
"""
print(json.dumps(parsed_issues, indent=2))