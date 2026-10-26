# Agent Instructions

## Language

- Use English for all project communication, documentation, code comments, identifiers, examples, and user-facing messages stored in the repository.
- Keep established names and syntax from external projects unchanged when required.

## Implementation

- Prefer existing project functions and established libraries or crates when they meet the need. Do not recreate functionality without a clear project-specific reason.
- Reuse existing functions for their intended purpose instead of duplicating logic.
- Write clean, secure, maintainable code. Validate inputs at appropriate boundaries and avoid unsafe code unless it is necessary and justified.
- Keep source files under 500 lines. Split responsibilities into focused modules when a file approaches that limit. Exempt generated or vendored files where splitting is not appropriate.
- Make the smallest coherent change that fully addresses the task.

## Commits

- Keep each commit atomic and limited to one logical change.
- Use Conventional Commit subjects in this format: `<type>(<optional scope>): <imperative summary>`.
- Use an appropriate type, such as `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, or `chore`.
- For every agent-authored commit, include these lines in the commit body:

  ```text
  Agent-Authored: yes
  Agent: <agent name and model>
  ```

- Describe the change clearly in the subject and body. Do not combine unrelated work in one commit.
