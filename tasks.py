import re
import json

markdown_milestone = """
# Rustis v1.0

## 1. Comprehensive Logging Infrastructure

Estimate: 1 day
Labels: ops, logging, v1.0
Priority: Critical

Objective: We should have a great, configurable logging system for tracking problems, seeing what the server is doing, and keeping a record of important events.

Sub-Tasks

- [ ] Integrate a modern, structured logging library (like tracing) and get it configured.
- [ ] Set up logging levels (Debug, Info, Warn, Error) that we can control easily, maybe with a config file or environment variables.
- [ ] Make sure we're logging key events, like:
  - [ ] Server starting and stopping.
  - [ ] Clients connecting and disconnecting.
  - [ ] Command execution (maybe only at the DEBUG level).
  - [ ] Errors and any warnings (especially if they involve concurrency or locking!).
- [ ] The log output needs to be clean and easy for machines to read (like JSON or key-value pairs).

## 2. Performance Counters and Lock Metrics

Estimate: 4 days
Labels: observability, monitoring, performance, v1.0
Priority: Critical

Objective: We need comprehensive metrics so we can see exactly what's happening inside the server, especially around locking! We want to spot any lock contention immediately.

Sub-Tasks

- [ ] Integrate a metrics library that works with tools like Prometheus.
- [ ] Instrument the code to track simple stuff like commands_processed, bytes_read, and bytes_written.
- [ ] Implement specific lock-related metrics:
  - [ ] How long are people waiting for a lock? (lock_wait_duration_seconds histogram).
  - [ ] How often do lock attempts fail? (lock_acquire_failures counter).
  - [ ] How many locks are currently active? (active_locks gauge).
- [ ] Set up a simple /metrics endpoint so external monitoring tools can scrape the data.

## 3. Implement REDIS-like Text Protocol (RESP)

Estimate: 1 week
Labels: protocol, networking, v1.0
Priority: Critical

Objective: We need a solid parser and serializer for the Redis Serialization Protocol (RESP) so all the standard Redis clients can talk to our server easily. Compatibility is key!

Sub-Tasks

- [ ] Read and propose a subset of RESP to implement
  - [ ] Define the subset of VERBS to implement.
  - [ ] Define the data types for RESP (Simple Strings, Errors, Integers, Bulk Strings, Arrays).
- [ ] Implement a fast, non-blocking RESP parser that handles partial socket reads gracefully.
- [ ] Implement the RESP serializer.
- [ ] Implement missing VERBS.

## 4. Key-Level Locking Mechanism

Estimate: 1 week
Labels: concurrency, data-store, v1.0
Priority: High

Objective: We've gotta make sure data access is totally safe for multi-threading! We'll use smart, granular locking for each individual key instead of one huge, slow lock for the whole database. No global bottlenecks here!

Sub-Tasks

- [ ] Research and test key-level locking. Write up a brief describing how it works.
- [ ] Review other Rust concurrency patterns to make sure we're keeping lock times as short as possible.
- [ ] Develop some serious stress tests to confirm the safety and see how fast our key-level locking is when things get busy.

## 5. Implement Alternative Binary Protocol

Estimate: 1 week
Labels: protocol, performance, v1.0
Priority: Low (stretch goal)

Objective: Time to build our own super-efficient binary protocol! This one should be even faster than RESP, with way less overhead, for maximum performance.

Sub-Tasks

- [ ] Decide on the base binary format for the binary protocol
  - [ ] Msgpack
  - [ ] Protobuf
  - [ ] Other?
- [ ] Design the structure for this new binary protocol (like message headers and how data is encoded).
- [ ] Implement a dedicated parser and serializer just for this binary format.
- [ ] Add Configuration so that listeners can be define by address, port and protocol.
- [ ] Make sure to write some performance tests comparing how fast RESP and Binary serialization/deserialization really are!
"""

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
    
    all_issues = []
    
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
            line = line.strip()
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
        subtasks = []
        current_parent_title = None
        current_subtask_body = []
        
        # Combine the rest of the lines into a single string for easier regex parsing of tasks
        content = "\n".join(content_lines)
        
        # Regex to find all top-level and nested checklist items
        # Group 1: indent (0 or 2 spaces)
        # Group 2: checkbox status ([ ] or [x])
        # Group 3: task text
        task_regex = re.compile(r'^\s*(-)\s+\[[ x]\]\s*(.+)', re.MULTILINE)
        
        # This will hold the structured sub-issues
        parsed_sub_issues = []
        
        for line in content.split('\n'):
            match = task_regex.match(line)
            if not match:
                continue

            indent = line.split('-')[0]
            task_text = match.group(2).strip()
            
            if len(indent) == 0:
                # Top-level task: Start a new GitHub Issue (Subtask)
                
                # If there was a previous parent, save it now
                if current_parent_title:
                    # Finalize the previous subtask
                    parsed_sub_issues.append({
                        "title": current_parent_title,
                        "body": "\n".join(current_subtask_body).strip(),
                        "iteration": iteration_name,
                        "labels": metadata.get('labels', '') # Inherit labels from parent
                    })
                
                # Start a new subtask
                current_parent_title = f"{main_issue_title}: {task_text}"
                current_subtask_body = []
                
            elif len(indent) >= 2:
                # Nested task: Add to the body of the *current* subtask
                # Reformat the nested item as a clean checklist item for the body
                current_subtask_body.append(f"- [ ] {task_text}")


        # Finalize the last subtask
        if current_parent_title:
            parsed_sub_issues.append({
                "title": current_parent_title,
                "body": "\n".join(current_subtask_body).strip(),
                "iteration": iteration_name,
                "labels": metadata.get('labels', '')
            })

        # 3. Create the Parent Epic Issue
        parent_issue = {
            "title": f"EPIC: {main_issue_title}",
            "body": f"**Objective:** {metadata.get('objective', 'N/A')}\n\n**Estimate:** {metadata.get('estimate', 'N/A')}\n**Priority:** {metadata.get('priority', 'N/A')}\n\n---",
            "iteration": iteration_name, # Parent also belongs to the iteration
            "labels": metadata.get('labels', '') + ", epic",
            "subtasks": parsed_sub_issues # Add the subtasks for tracking
        }
        
        all_issues.append(parent_issue)
    
    return all_issues

# --- Execution ---

parsed_issues = parse_milestone(markdown_milestone)

print("✅ Successfully parsed milestone into structured issues. ")
print("\n--- Summary of Issues to Create ---")

for issue in parsed_issues:
    print(f"\n--- PARENT ISSUE/EPIC ---")
    print(f"TITLE: {issue['title']}")
    print(f"ITERATION: {issue['iteration']}")
    print(f"LABELS: {issue['labels']}")
    print(f"SUBTASK COUNT: {len(issue['subtasks'])}")

    for subtask in issue['subtasks']:
        print(f"  - SUBTASK TITLE: {subtask['title']}")
        if subtask['body']:
            print(f"    - Body (Nested Checklist Items): \n      {subtask['body'].replace('\n', '\n      ')}")

print("\n--- Full JSON Output (Simplified) ---")
print(json.dumps(parsed_issues, indent=2))