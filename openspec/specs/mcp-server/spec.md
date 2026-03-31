# MCP Server Specification

## Purpose

Expose the game to AI agents via MCP (Streamable HTTP). Agents authenticate with per-match Bearer tokens and interact through 5 tools and 3 resources. All tool responses are scoped to the player's fog of war.

## Requirements

### Requirement: Agent authentication

The MCP server SHALL authenticate agents via Bearer tokens.

#### Scenario: Valid token

- GIVEN a valid agent_token for an active match
- WHEN a request arrives with Authorization: Bearer <token>
- THEN resolve the token to a match_player
- AND allow MCP tool/resource access scoped to that player's slot

#### Scenario: Invalid token

- GIVEN an unrecognized token
- WHEN a request arrives
- THEN reject with 401

#### Scenario: Inactive match

- GIVEN a token for a match not in ACTIVE status
- WHEN a request arrives
- THEN reject with 400

### Requirement: get_game_state tool

The server SHALL return the visible game state for the authenticated player.

#### Scenario: Normal state query

- GIVEN an authenticated agent
- WHEN get_game_state is called
- THEN return tick, energy, visible units, buildings, resources, and map dimensions
- AND filter all entities by the player's fog of war

### Requirement: spawn_unit tool

The server SHALL queue unit spawn commands.

#### Scenario: Valid spawn

- GIVEN sufficient energy for the unit type
- WHEN spawn_unit is called with a valid unit_type
- THEN queue the command for the next tick
- AND return status: queued

#### Scenario: Insufficient energy

- GIVEN insufficient energy
- WHEN spawn_unit is called
- THEN return an error with cost and current energy

### Requirement: move_unit tool

The server SHALL queue move commands.

#### Scenario: Valid move

- GIVEN a unit_id owned by the player and valid coordinates
- WHEN move_unit is called
- THEN queue the command
- AND return status: queued

### Requirement: attack_target tool

The server SHALL queue attack commands.

#### Scenario: Valid attack

- GIVEN a unit_id owned by the player and a target_id
- WHEN attack_target is called
- THEN queue the command
- AND return status: queued

### Requirement: harvest tool

The server SHALL queue harvest commands.

#### Scenario: Valid harvest

- GIVEN a worker unit_id and a resource_id
- WHEN harvest is called
- THEN queue the command
- AND return status: queued

### Requirement: Game resources

The server SHALL expose static/semi-static game information via MCP resources.

#### Scenario: Rules resource

- GIVEN game://rules is requested
- THEN return full game rules as markdown (unit stats, costs, win conditions)

#### Scenario: Map topology resource

- GIVEN game://map/topology is requested
- THEN return map dimensions and start positions as JSON

#### Scenario: Match status resource

- GIVEN game://match/status is requested
- THEN return phase, tick, max ticks, winner, and player energy levels
