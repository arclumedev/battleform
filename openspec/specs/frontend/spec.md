# Frontend Specification

## Purpose

Vue 3 shell application that handles authentication, lobby management, and mounts the WASM game client. Provides overlays for command log and stats during matches.

## Requirements

### Requirement: Login

The frontend SHALL provide email/password and OAuth login options.

#### Scenario: Email login

- GIVEN the login page
- WHEN the user enters email and password and submits
- THEN POST to /api/auth/login
- AND on success, redirect to the lobby

#### Scenario: Email registration

- GIVEN the login page in register mode
- WHEN the user submits email, password, and optional name
- THEN POST to /api/auth/register
- AND on success, redirect to the lobby

#### Scenario: OAuth login

- GIVEN the login page
- WHEN the user clicks Google or GitHub
- THEN redirect to the OAuth provider's authorization URL

### Requirement: Lobby

The frontend SHALL display matches and allow creation/joining.

#### Scenario: Match list

- GIVEN the lobby page
- WHEN it loads
- THEN fetch and display all matches with status, players, and tick count

#### Scenario: Quick play

- GIVEN the lobby page
- WHEN the user clicks a quick play button (1v1, 1v3, 1v7)
- THEN POST to /api/matches/quick with the player count
- AND navigate to the match view

#### Scenario: Create lobby

- GIVEN the lobby page
- WHEN the user clicks Create Lobby
- THEN POST to /api/matches
- AND navigate to the match view

### Requirement: Match view

The frontend SHALL mount the game client and display overlays.

#### Scenario: Active match

- GIVEN an active match
- WHEN the match view loads
- THEN mount the WASM game client canvas
- AND connect the WebSocket spectator bridge
- AND display the stats panel and command log

#### Scenario: Lobby phase

- GIVEN a match in LOBBY status
- WHEN the match view loads
- THEN show join/start buttons and player list
- AND do not mount the game client

#### Scenario: Finished match

- GIVEN a finished match
- WHEN the match view loads
- THEN display the winner

### Requirement: Stats panel

The frontend SHALL display real-time match statistics.

#### Scenario: Player stats

- GIVEN an active match with connected players
- WHEN stats are displayed
- THEN show each player's name, model, connection status, and slot color

### Requirement: Command log

The frontend SHALL display a scrolling feed of MCP tool calls.

#### Scenario: Log display

- GIVEN commands being issued during a match
- WHEN the log updates
- THEN show tick number, player, tool name, and arguments
- AND auto-scroll to the latest entry
