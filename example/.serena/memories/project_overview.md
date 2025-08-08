# GameHub Project Overview

## Purpose
GameHub is a blockchain-based gaming hub application built on the Linera SDK. It manages players, games, leaderboards, scoring systems, and moderation features for competitive gaming events with Discord integration.

## Tech Stack
- **Language**: Rust (Edition 2021)
- **Blockchain Framework**: Linera SDK 0.14.1
- **GraphQL**: async-graphql 7.0
- **Serialization**: serde with JSON support
- **Async Runtime**: tokio (for tests)
- **Validation**: regex patterns
- **Error Handling**: thiserror

## Architecture
- **Core Layer**: Business logic (types, domain services, validation)
- **Infrastructure Layer**: Blockchain state, messaging, operations
- **API Layer**: GraphQL interfaces
- **Test Layer**: Organized unit and integration tests

## Key Features
- Player registration and management
- Game approval workflow
- Scoring with streak multipliers
- Leaderboard processing
- Moderation (bans, suspensions)
- Permission system (admins, moderators)
- Cross-chain messaging
- Audit logging