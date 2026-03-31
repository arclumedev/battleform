# Spectator Specification

## Purpose

Broadcast real-time game state to browser spectators via WebSocket. State is serialized as MessagePack for minimal latency.

## Requirements

### Requirement: WebSocket connection

The system SHALL accept WebSocket connections for match spectating.

#### Scenario: Successful upgrade

- GIVEN a request to GET /api/matches/:id/spectate
- WHEN the HTTP connection upgrades to WebSocket
- THEN accept the connection and track it for the match

### Requirement: Initial snapshot

The system SHALL send a full state snapshot on connect.

#### Scenario: New spectator

- GIVEN a spectator connects to an active match
- WHEN the connection is established
- THEN send a MessagePack-encoded full state snapshot (units, buildings, resources, players, map)

### Requirement: Tick broadcast

The system SHALL broadcast state diffs each tick.

#### Scenario: Normal tick

- GIVEN connected spectators
- WHEN the game engine produces a state diff
- THEN broadcast the MessagePack-encoded diff to all spectators of that match

### Requirement: Match finish

The system SHALL notify spectators when a match ends.

#### Scenario: Match finished

- GIVEN connected spectators
- WHEN the match ends
- THEN broadcast a finish message with the winner slot
- AND close all spectator connections for that match

### Requirement: Connection cleanup

The system SHALL handle spectator disconnects gracefully.

#### Scenario: Spectator disconnects

- GIVEN a spectator closes their connection
- WHEN the WebSocket closes
- THEN remove the connection from the match's spectator set
