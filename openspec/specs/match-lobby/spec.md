# Match Lobby Specification

## Purpose

Manage match creation, player joining, and match lifecycle. Supports 2-8 players with any mix of agents (external MCP clients) and autopilots (server-side bots).

## Requirements

### Requirement: Match creation

The system SHALL allow authenticated users to create matches.

#### Scenario: Create default lobby

- GIVEN an authenticated user
- WHEN POST /api/matches is called
- THEN create a match in LOBBY status with 2 max_players
- AND generate a default 32x32 map config

#### Scenario: Create configured lobby

- GIVEN an authenticated user with slot configuration
- WHEN POST /api/matches/configured is called with max_players and slot types
- THEN create a match with the specified player count
- AND pre-create autopilot player records for autopilot slots
- AND generate a map config sized for the player count

### Requirement: Quick play

The system SHALL provide instant matches against autopilots.

#### Scenario: Quick play 1v1

- GIVEN an authenticated user
- WHEN POST /api/matches/quick is called with player_count=2
- THEN create a match in ACTIVE status
- AND join the user as slot 0 (agent)
- AND create an autopilot in slot 1
- AND start the game engine with autopilot on slot 1
- AND return the user's agent token

#### Scenario: Quick play FFA

- GIVEN an authenticated user requesting player_count=4
- WHEN POST /api/matches/quick is called
- THEN create 3 autopilots in slots 1-3
- AND start the game engine with autopilots on slots 1-3

### Requirement: Join match

The system SHALL allow users to join lobby matches.

#### Scenario: Join available match

- GIVEN a LOBBY match with open agent slots
- WHEN POST /api/matches/:id/join is called
- THEN assign the next available slot
- AND generate and return an agent token

#### Scenario: Match full

- GIVEN a match at max_players capacity
- WHEN a join is attempted
- THEN reject with 400

#### Scenario: Already joined

- GIVEN a user already in the match
- WHEN they attempt to join again
- THEN reject with 400

### Requirement: Start match

The system SHALL allow the match creator to start when all slots are filled.

#### Scenario: Successful start

- GIVEN a LOBBY match with all slots filled
- WHEN the creator calls POST /api/matches/:id/start
- THEN set status to ACTIVE
- AND initialize the game engine with autopilot slots
- AND start the tick loop

#### Scenario: Not enough players

- GIVEN a match with empty slots
- WHEN start is attempted
- THEN reject with 400

#### Scenario: Non-creator start

- GIVEN a user who is not the match creator
- WHEN they attempt to start
- THEN reject with 403
