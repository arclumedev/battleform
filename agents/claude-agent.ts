import Anthropic from "@anthropic-ai/sdk";

// ---------------------------------------------------------------------------
// Environment
// ---------------------------------------------------------------------------
const AGENT_TOKEN = process.env.AGENT_TOKEN;
if (!AGENT_TOKEN) {
  console.error("AGENT_TOKEN env var is required");
  process.exit(1);
}

const API_BASE = process.env.API_BASE ?? "http://localhost:3333";
const MODEL = process.env.MODEL ?? "claude-sonnet-4-20250514";

// ---------------------------------------------------------------------------
// Tool definitions (Anthropic format)
// ---------------------------------------------------------------------------
const TOOLS: Anthropic.Tool[] = [
  {
    name: "get_game_state",
    description:
      "Returns the full current game state including the map, all units, resources, and whose turn it is.",
    input_schema: {
      type: "object" as const,
      properties: {},
      required: [],
    },
  },
  {
    name: "spawn_unit",
    description:
      "Spawn a new unit at the given coordinates. Costs resources. unit_type is e.g. 'warrior', 'archer', 'harvester'.",
    input_schema: {
      type: "object" as const,
      properties: {
        unit_type: { type: "string", description: "Type of unit to spawn" },
        x: { type: "number", description: "X coordinate" },
        y: { type: "number", description: "Y coordinate" },
      },
      required: ["unit_type", "x", "y"],
    },
  },
  {
    name: "move_unit",
    description: "Move an existing unit to a new position on the map.",
    input_schema: {
      type: "object" as const,
      properties: {
        unit_id: { type: "string", description: "ID of the unit to move" },
        x: { type: "number", description: "Target X coordinate" },
        y: { type: "number", description: "Target Y coordinate" },
      },
      required: ["unit_id", "x", "y"],
    },
  },
  {
    name: "attack_target",
    description: "Order a unit to attack an enemy unit.",
    input_schema: {
      type: "object" as const,
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
  {
    name: "harvest",
    description:
      "Order a harvester unit to gather resources from its current tile.",
    input_schema: {
      type: "object" as const,
      properties: {
        unit_id: {
          type: "string",
          description: "ID of the harvester unit",
        },
      },
      required: ["unit_id"],
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
): Promise<unknown> {
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
  const client = new Anthropic();

  console.log(`Battleform Claude Agent`);
  console.log(`  Model : ${MODEL}`);
  console.log(`  Server: ${API_BASE}`);
  console.log();

  const messages: Anthropic.MessageParam[] = [];
  let turnCount = 0;

  // Kick off with a user message asking the agent to play
  messages.push({
    role: "user",
    content:
      "A new game has started. You are a player in Battleform. Take your first turn by inspecting the game state and then making your moves.",
  });

  while (true) {
    turnCount++;
    console.log(`\n=== Turn ${turnCount} ===`);

    const response = await client.messages.create({
      model: MODEL,
      max_tokens: 4096,
      system: SYSTEM_PROMPT,
      tools: TOOLS,
      messages,
    });

    // Collect assistant content blocks
    const assistantContent = response.content;
    messages.push({ role: "assistant", content: assistantContent });

    // Check for game over in text blocks
    const textBlocks = assistantContent.filter(
      (b): b is Anthropic.TextBlock => b.type === "text"
    );
    for (const block of textBlocks) {
      if (block.text) console.log(`  Agent: ${block.text.slice(0, 300)}`);
    }

    // If the model didn't request any tool use, the turn is over
    const toolUseBlocks = assistantContent.filter(
      (b): b is Anthropic.ToolUseBlock => b.type === "tool_use"
    );

    if (toolUseBlocks.length === 0) {
      console.log("  (no tool calls — turn complete)");

      // Check for game over
      const fullText = textBlocks.map((b) => b.text).join(" ");
      if (fullText.includes('"phase":"finished"') || fullText.includes("game over")) {
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
    const toolResults: Anthropic.ToolResultBlockParam[] = [];

    for (const toolUse of toolUseBlocks) {
      try {
        const result = await callMcpTool(
          toolUse.name,
          (toolUse.input as Record<string, unknown>) ?? {}
        );
        const resultStr = typeof result === "string" ? result : JSON.stringify(result);

        // Check for game over in tool results
        if (resultStr.includes('"phase":"finished"')) {
          console.log("\nGame over detected in tool result. Exiting.");
          toolResults.push({
            type: "tool_result",
            tool_use_id: toolUse.id,
            content: resultStr,
          });
          messages.push({ role: "user", content: toolResults });
          return;
        }

        toolResults.push({
          type: "tool_result",
          tool_use_id: toolUse.id,
          content: resultStr,
        });
      } catch (err) {
        console.error(`  !! Tool error: ${err}`);
        toolResults.push({
          type: "tool_result",
          tool_use_id: toolUse.id,
          content: `Error: ${err}`,
          is_error: true,
        });
      }
    }

    messages.push({ role: "user", content: toolResults });

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
