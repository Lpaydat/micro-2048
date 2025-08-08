# GameHub Code Style and Conventions

## File Structure
- **Modular Architecture**: Clean separation between core, infrastructure, and API layers
- **Domain-Driven Design**: Services organized by business domain
- **Test Organization**: Tests mirror source structure

## Naming Conventions
- **Modules**: snake_case (e.g., `player_service.rs`)
- **Structs/Enums**: PascalCase (e.g., `GameHubState`, `PlayerStatus`)
- **Functions**: snake_case (e.g., `validate_discord_id`)
- **Constants**: SCREAMING_SNAKE_CASE (e.g., `MAX_USERNAME_LENGTH`)

## Documentation
- Copyright headers on all files
- Module-level documentation with `//!`
- Function documentation with `///` for public APIs
- Inline comments for complex business logic

## Error Handling
- Custom error types using `thiserror`
- Result types for all fallible operations
- Detailed error messages with context

## Async Patterns
- `async fn` for all I/O operations
- Proper error propagation with `?`
- Use of `Timestamp` from Linera SDK

## Imports Organization
1. Standard library imports
2. External crate imports  
3. Internal crate imports (using relative paths)
4. Re-exports at module level