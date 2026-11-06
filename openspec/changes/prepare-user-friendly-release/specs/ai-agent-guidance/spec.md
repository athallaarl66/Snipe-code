## Purpose

Defines what generated AI-agent instructions and commands tell users and agents to do, so heuristic audit results trigger review rather than unverified automatic edits.

## ADDED Requirements

### Requirement: Agent guidance treats audit findings as unverified leads
Generated agent rules, skills, and slash commands SHALL require review of each HIGH finding against the cited source before proposing a change. They SHALL NOT direct an agent to edit code solely because a heuristic finding is HIGH.

#### Scenario: False positive
- **WHEN** an audit reports HIGH for safe code
- **THEN** the agent guidance calls for explaining the false positive and leaving that code unchanged

#### Scenario: Confirmed risk
- **WHEN** the finding is confirmed against source behavior
- **THEN** guidance calls for the smallest safe fix, project tests, a repeat audit, and a report of what was verified

### Requirement: Generated AI support is opt-in and independently usable
The CLI SHALL generate dedicated agent artifacts only for selected tool identifiers; the shared project guidance SHALL remain available when no AI tool is selected. Documentation SHALL distinguish CLI commands from agent slash commands and SHALL state that agent commands requiring `snipe-code` need the executable on PATH.

#### Scenario: No AI tool selected
- **WHEN** a project is generated without any selected AI tool
- **THEN** it has shared guidance but no tool-specific instruction, skill, or slash-command artifact

#### Scenario: Slash command used without CLI
- **WHEN** an agent slash command attempts an audit and `snipe-code` is not on PATH
- **THEN** guidance states the installation prerequisite rather than claiming the agent tool embeds the CLI
