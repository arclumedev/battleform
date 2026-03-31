# Game Engine Specification

## Purpose

Server-side authoritative game engine. Processes commands, resolves combat and harvesting, computes fog of war, checks win conditions, and generates state diffs for spectators.

## Requirements

### Requirement: Tick loop

The engine SHALL process game state at 10 ticks per second.

#### Scenario: Normal tick

- GIVEN an active match
- WHEN a tick fires (every 100ms)
- THEN drain the command queue
- AND execute commands
- AND resolve movement
- AND resolve combat
- AND resolve harvesting
- AND compute fog of war
- AND check win conditions
- AND generate and broadcast a state diff

### Requirement: Unit spawning

The engine SHALL create units at the player's base when they have sufficient energy.

#### Scenario: Spawn with enough energy

- GIVEN a player with 200 energy
- WHEN spawn_unit(soldier) is commanded (cost: 100)
- THEN deduct 100 energy
- AND create a soldier at the player's base position

#### Scenario: Spawn with insufficient energy

- GIVEN a player with 50 energy
- WHEN spawn_unit(soldier) is commanded
- THEN do nothing and leave energy unchanged

### Requirement: Movement

The engine SHALL move units along A* paths on the grid.

#### Scenario: Move to open tile

- GIVEN an idle unit and an open destination
- WHEN move_unit is commanded
- THEN compute an A* path
- AND set unit status to moving
- AND advance the unit along the path each tick (up to speed tiles per tick)

#### Scenario: Blocked destination

- GIVEN a blocked destination tile
- WHEN move_unit is commanded
- THEN do nothing (no path found)

### Requirement: Combat

The engine SHALL resolve combat simultaneously each tick.

#### Scenario: Melee attack in range

- GIVEN an attacking unit adjacent to its target
- WHEN combat resolves
- THEN apply the attacker's damage to the target
- AND generate a combat event

#### Scenario: Simultaneous damage

- GIVEN two units attacking each other
- WHEN combat resolves
- THEN both take damage (no first-strike advantage)

#### Scenario: Unit death

- GIVEN a unit reduced to 0 or fewer HP
- WHEN damage is applied
- THEN remove the unit from the game state

### Requirement: Harvesting

The engine SHALL allow workers to collect energy from resource nodes.

#### Scenario: Harvest and deposit

- GIVEN a worker adjacent to a resource node with remaining energy
- WHEN harvesting resolves
- THEN transfer 25 energy from the node to the worker's cargo
- AND set status to returning
- AND path the worker back to base

#### Scenario: Deposit at base

- GIVEN a returning worker adjacent to its base with cargo
- WHEN harvesting resolves
- THEN add cargo to the player's energy pool
- AND clear the worker's cargo
- AND set status to idle

### Requirement: Fog of war

The engine SHALL compute per-player visibility based on unit and building vision ranges.

#### Scenario: Unit vision

- GIVEN a soldier with vision radius 3 at position (8,8)
- WHEN fog is computed
- THEN tiles within radius 3 of (8,8) are visible for that player

#### Scenario: Scout extended vision

- GIVEN a scout with vision radius 6
- WHEN fog is computed
- THEN tiles within radius 6 are visible (2x normal)

#### Scenario: Fog downgrade

- GIVEN a tile that was visible last tick but no longer in any unit's range
- WHEN fog is computed
- THEN the tile becomes previously_seen (not unseen)

### Requirement: Win conditions

The engine SHALL end matches when a win condition is met.

#### Scenario: Last base standing

- GIVEN a match with N players
- WHEN only one player's base remains
- THEN end the match with that player as winner

#### Scenario: Max ticks score

- GIVEN a match that reaches max ticks with multiple bases alive
- WHEN the tick limit is reached
- THEN score each player (base HP + unit count * 10 + energy)
- AND the highest score wins (null for tie)

### Requirement: Autopilot AI

The engine SHALL generate commands for autopilot slots every 5 ticks.

#### Scenario: Early game

- GIVEN an autopilot with no workers
- WHEN commands are generated
- THEN spawn workers for economy

#### Scenario: Scouting

- GIVEN an autopilot with idle scouts
- WHEN commands are generated
- THEN move scouts toward the nearest enemy start position

#### Scenario: Attack

- GIVEN an autopilot with idle soldiers and a visible enemy base
- WHEN commands are generated
- THEN attack the nearest enemy base
