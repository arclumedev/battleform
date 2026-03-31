# Agents Specification

## Purpose

Agent harness scripts that connect AI models (Claude, GPT) to matches via MCP. Each agent authenticates with a Bearer token, queries game state, and issues commands in a loop.

## Requirements

### Requirement: Agent connection

Agent harnesses SHALL connect to the MCP server with a Bearer token.

#### Scenario: Successful connection

- GIVEN a valid AGENT_TOKEN environment variable
- WHEN the agent starts
- THEN authenticate via Bearer token on each MCP request

### Requirement: Game loop

Agent harnesses SHALL run a continuous game loop using LLM tool calls.

#### Scenario: Turn cycle

- GIVEN an active match
- WHEN the agent's turn runs
- THEN call get_game_state to assess the situation
- AND use the LLM to decide which commands to issue
- AND call spawn_unit, move_unit, attack_target, or harvest as needed
- AND repeat until the game ends

### Requirement: Claude agent

The Claude agent SHALL use the Anthropic SDK with tool definitions.

#### Scenario: Claude tool use

- GIVEN the game tools defined as Anthropic tool schemas
- WHEN Claude responds with tool_use blocks
- THEN execute each tool call against the MCP server
- AND feed results back as tool_result messages

### Requirement: OpenAI agent

The OpenAI agent SHALL use the OpenAI SDK with function calling.

#### Scenario: GPT tool use

- GIVEN the game tools defined as OpenAI function schemas
- WHEN GPT responds with tool_calls
- THEN execute each function call against the MCP server
- AND feed results back as tool messages

### Requirement: Game over handling

Agent harnesses SHALL detect match completion and exit cleanly.

#### Scenario: Match finished

- GIVEN a tool response containing phase: "finished"
- WHEN the agent processes the response
- THEN log the result and exit
