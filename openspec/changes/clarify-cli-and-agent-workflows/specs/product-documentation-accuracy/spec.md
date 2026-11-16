## ADDED Requirements

### Requirement: Documentation separates product execution from agent assistance
User-facing documentation SHALL distinguish direct executable commands, generated project-local AI artifacts, invocable agent workflows, and auto-loaded instructions or skills.

#### Scenario: Direct CLI documentation
- **WHEN** documentation describes `new`, `analyze`, `audit`, or `verify`
- **THEN** it states that the executable can run without an AI model and identifies filesystem, network, and external-tool side effects

#### Scenario: Agent workflow documentation
- **WHEN** documentation lists a supported AI agent
- **THEN** it identifies whether that agent receives rules, a skill, or an invocable command or workflow and does not claim unsupported slash-command behavior

### Requirement: Documentation describes test ownership and report limits
Documentation SHALL state that generated projects include example test infrastructure, verification runs existing declared tests, AI-agent workflows may author tests, and verification report formats are JSON, JUnit XML, and Markdown.

#### Scenario: Test generation claim
- **WHEN** documentation explains automated testing
- **THEN** it does not claim that the standalone executable infers complete test cases from source code

#### Scenario: Report format claim
- **WHEN** documentation lists report formats
- **THEN** it separates audit Markdown, PDF, DOCX, and JSON output from verification JSON, JUnit XML, and Markdown output

### Requirement: Documentation reflects the English interactive flow
README and command help SHALL show the English intent launcher, generation steps, destination summary, confirmation choices, and direct-command bypass.

#### Scenario: First-time executable use
- **WHEN** a user reads how to open the executable without arguments
- **THEN** the documented first screen matches the launcher choices and does not show the template prompt as the product entry point

#### Scenario: Experienced user path
- **WHEN** a user wants scripting or CI usage
- **THEN** documentation shows direct commands that bypass the launcher
