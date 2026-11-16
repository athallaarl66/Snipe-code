## ADDED Requirements

### Requirement: Blog preset describes its RSS behavior truthfully
A generated blog project SHALL expose its RSS output as a sample snapshot if the tool does not update the feed when posts change. Product and generated-project documentation SHALL NOT describe that static file as an automatically updated feed.

#### Scenario: New post added after generation
- **WHEN** the user adds a post after generating the blog preset and no RSS update mechanism is present
- **THEN** documentation explains that the user must update or replace the sample feed, rather than promising that the new post appears automatically

#### Scenario: Sample URL
- **WHEN** the sample feed contains a localhost URL
- **THEN** generated guidance marks it as a placeholder to change before deployment
