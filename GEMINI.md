# Gemini Project Context

Always refer to [CLAUDE.md](./CLAUDE.md) for project-level memory, architecture, commands, and constraints. This file serves as the primary source of truth for the development environment and standards.

## General Behavior
- Never tell the user you're giving up on a solution. Instead tell the user you're stuck and need further instructions to continue. 
- Never revert changes on your own. The user will handle reverting changes if they deem it's necessary.
- Always plan first and review the plan with the user before asking permission to execute the plan.
- Always break down work into small, testable steps. No step is done before successful testing is completed.
