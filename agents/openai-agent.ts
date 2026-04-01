import OpenAI from "openai";
import type { ChatCompletionMessageParam, ChatCompletionTool } from "openai/resources/chat/completions";

// ---------------------------------------------------------------------------
// Environment
// ---------------------------------------------------------------------------
const AGENT_TOKEN = process.env.AGENT_TOKEN;
if (!AGENT_TOKEN) {
  console.error("AGENT_TOKEN env var is required");
  process.exit(1);
}

const API_BASE = process.env.API_BASE ?? "http://localhost:3333";
const MODEL = process.env.MODEL ?? "gpt-4o";

// ---------------------------------------------------------------------------
// Tool definitions (OpenAI function-calling format)
// ---------------------------------------------------------------------------
const TOOLS: ChatCompletionTool[] = [
  {
    type: "function",
    function: {
      name: "get_game_state",
      description:
        "Returns the full current game state including the map, all units, resources, and whose turn it is.",
      parameters: {
        type: "object",
        properties: {},
        required: [],
      },
    },
  },
  {
    type: "function",
    function: {
      name: "spawn_unit",
      description:
        "Spawn a new unit at the given coordinates. Costs resources. unit_type is e.g. 'warrior', 'archer', 'harvester'.",
      parameters: {
        type: "object",
        properties: {
          unit_type: { type: "string", description: "Type of unit to spawn" },
          x: { type: "number", description: "X coordinate" },
          y: { type: "number", description: "Y coordinate" },
        },
        required: ["unit_type", "x", "y"],
      },
    },
  },
  {
    type: "function",
    function: {
      name: "move_unit",
      description: "Move an existing unit to a new position on the map.",
      parameters: {
        type: "object",
        properties: {
          unit_id: { type: "string", description: "ID of the unit to move" },
          x: { type: "number", description: "Target X coordinate" },
          y: { type: "number", description: "Target Y coordinate" },
        },
        required: ["unit_id", "x", "y"],
      },
    },
  },
  {
    type: "function",
    function: {
      name: "attack_target",
      description: "Order a unit to attack an enemy unit.",
      parameters: {
        type: "object",
        properties: {
          unit_id: {
            type: "string",
            description: "ID of the attacking unit",
          },
          target_id: {
            type: "string",
            description: "ID of the enemy unit to attack",
          },
        },
        required: ["unit_id", "target_id"],
      },
    },
  },
  {
    type: "function",
    function: {
      name: "harvest",
      description:
        "Order a harvester unit to gather resources from its current tile.",
      parameters: {
        type: "object",
        properties: {
          unit_id: {
            type: "string",
            description: "ID of the harvester unit",
          },
        },
        required: ["unit_id"],
      },
    },
  },
];

// ---------------------------------------------------------------------------
// MCP JSON-RPC helper
// ---------------------------------------------------------------------------
let rpcId = 0;

async function callMcpTool(
  toolName: string,
  args: Record<string, unknown>
): Promise<string> {
  const body = {
    jsonrpc: "2.0",
    id: ++rpcId,
    method: "tools/call",
    params: { name: toolName, arguments: args },
  };

  console.log(`  -> MCP tool: ${toolName}`, JSON.stringify(args));

  const res = await fetch(`${API_BASE}/api/mcp`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${AGENT_TOKEN}`,
    },
    body: JSON.stringify(body),
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(`MCP request failed (${res.status}): ${text}`);
  }

  const json = (await res.json()) as {
    result?: { content?: Array<{ text?: string }> };
    error?: { message: string };
  };

  if (json.error) {
    throw new Error(`MCP error: ${json.error.message}`);
  }

  const content = json.result?.content?.[0]?.text ?? JSON.stringify(json.result);
  console.log(`  <- result: ${content.slice(0, 200)}`);
  return content;
}

// ---------------------------------------------------------------------------
// System prompt
// ---------------------------------------------------------------------------
const SYSTEM_PROMPT = `You are a competitive AI player in Battleform, a turn-based strategy game.

Your goal is to WIN the game by destroying all enemy units or controlling the map.

Strategy guidelines:
- Always start by calling get_game_state to understand the current situation.
- Gather resources early with harvesters to fund your army.
- Spawn a balanced mix of units: harvesters for economy, warriors for front-line combat, archers for ranged damage.
- Move units toward the enemy aggressively once you have a numbers advantage.
- Focus fire on wounded enemy units to eliminate them quickly.
- Protect your harvesters — keep them behind your combat units.
- Adapt your strategy based on what the enemy is doing.

Each turn you must decide which actions to take. Use the tools provided. After each tool call you will receive the result. When you are done with your turn, say "END_TURN".`;

// ---------------------------------------------------------------------------
// Game loop
// ---------------------------------------------------------------------------
async function main() {
  const client = new OpenAI();

  console.log(`Battleform OpenAI Agent`);
  console.log(`  Model : ${MODEL}`);
  console.log(`  Server: ${API_BASE}`);
  console.log();

  const messages: ChatCompletionMessageParam[] = [
    { role: "system", content: SYSTEM_PROMPT },
    {
      role: "user",
      content:
        "A new game has started. You are a player in Battleform. Take your first turn by inspecting the game state and then making your moves.",
    },
  ];

  let turnCount = 0;

  while (true) {
    turnCount++;
    console.log(`\n=== Turn ${turnCount} ===`);

    const response = await client.chat.completions.create({
      model: MODEL,
      max_tokens: 4096,
      tools: TOOLS,
      messages,
    });

    const choice = response.choices[0];
    if (!choice) {
      console.error("No choice returned from API");
      break;
    }

    const message = choice.message;
    messages.push(message);

    // Log any text content
    if (message.content) {
      console.log(`  Agent: ${message.content.slice(0, 300)}`);
    }

    // If no tool calls, the turn is over
    const toolCalls = message.tool_calls;
    if (!toolCalls || toolCalls.length === 0) {
      console.log("  (no tool calls — turn complete)");

      // Check for game over
      const text = message.content ?? "";
      if (text.includes('"phase":"finished"') || text.includes("game over")) {
        console.log("\nGame over detected. Exiting.");
        break;
      }

      // Prompt for next turn
      messages.push({
        role: "user",
        content:
          "It is now your turn again. Call get_game_state and decide your actions.",
      });

      await sleep(500);
      continue;
    }

    // Execute each tool call and feed results back
    for (const toolCall of toolCalls) {
      const fnName = toolCall.function.name;
      let args: Record<string, unknown> = {};
      try {
        args = JSON.parse(toolCall.function.arguments || "{}");
      } catch {
        console.error(`  !! Failed to parse args: ${toolCall.function.arguments}`);
      }

      try {
        const result = await callMcpTool(fnName, args);

        // Check for game over in tool results
        if (result.includes('"phase":"finished"')) {
          console.log("\nGame over detected in tool result. Exiting.");
          messages.push({
            role: "tool",
            tool_call_id: toolCall.id,
            content: result,
          });
          return;
        }

        messages.push({
          role: "tool",
          tool_call_id: toolCall.id,
          content: result,
        });
      } catch (err) {
        console.error(`  !! Tool error: ${err}`);
        messages.push({
          role: "tool",
          tool_call_id: toolCall.id,
          content: `Error: ${err}`,
        });
      }
    }

    await sleep(500);
  }
}

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

main().catch((err) => {
  console.error("Fatal error:", err);
  process.exit(1);
});
