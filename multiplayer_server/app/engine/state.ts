import crypto from 'node:crypto'

// --- Enums ---

export type TileType =
  | 'grass'
  | 'desert'
  | 'forest'
  | 'mountain'
  | 'water_lake'
  | 'water_sea'
  | 'snow'

export interface Tile {
  type: TileType
  elevation: number // 0-3
}

/** Movement cost per tile type. Infinity = impassable. */
export const TILE_MOVEMENT_COST: Record<TileType, number> = {
  grass: 1.0,
  desert: 1.5,
  forest: 1.5,
  mountain: Infinity,
  water_lake: Infinity,
  water_sea: Infinity,
  snow: 2.0,
}
export type UnitType = 'worker' | 'soldier' | 'scout'
export type UnitStatus = 'idle' | 'moving' | 'attacking' | 'harvesting' | 'returning' | 'dead'
export type BuildingType = 'base' | 'turret' | 'wall'
export type VisibilityState = 'unseen' | 'previously_seen' | 'visible'
export type MatchPhase = 'lobby' | 'active' | 'finished'

// --- Unit Stats ---

export const UNIT_STATS: Record<
  UnitType,
  { cost: number; hp: number; speed: number; range: number; damage: number; vision: number }
> = {
  worker: { cost: 50, hp: 30, speed: 2, range: 1, damage: 5, vision: 3 },
  soldier: { cost: 100, hp: 80, speed: 2, range: 1, damage: 20, vision: 3 },
  scout: { cost: 75, hp: 40, speed: 4, range: 1, damage: 10, vision: 6 },
}

export const BASE_STATS = { hp: 500, vision: 4 }
export const HARVEST_AMOUNT = 25
export const STARTING_ENERGY = 200

// --- Types ---

export interface Position {
  x: number
  y: number
}

export interface Unit {
  id: string
  playerSlot: number
  unitType: UnitType
  x: number
  y: number
  hp: number
  maxHp: number
  status: UnitStatus
  path: Position[]
  targetId: string | null
  cargo: number
}

export interface Building {
  id: string
  playerSlot: number
  buildingType: BuildingType
  x: number
  y: number
  hp: number
  maxHp: number
}

export interface ResourceNode {
  id: string
  x: number
  y: number
  remaining: number
}

export interface MapConfig {
  width: number
  height: number
  terrain: Tile[][] | string // Tile[][] for new maps, string for legacy compat
  resourceNodes: { x: number; y: number; energy: number }[]
  startPositions: Position[]
}

export interface PlayerState {
  slot: number
  energy: number
  baseId: string
}

export interface GameCommand {
  playerSlot: number
  toolName: string
  toolInput: Record<string, unknown>
}

// --- State Diff ---

export interface UnitMove {
  id: string
  x: number
  y: number
  status: UnitStatus
}

export interface CombatEvent {
  attackerId: string
  targetId: string
  damage: number
  x: number
  y: number
}

export interface StateDiff {
  tick: number
  unitsMoved: UnitMove[]
  unitsSpawned: Unit[]
  unitsKilled: string[]
  buildingsBuilt: Building[]
  buildingsDestroyed: string[]
  combatEvents: CombatEvent[]
  resourcesChanged: [number, number][]
  visibilityUpdates: { x: number; y: number; state: VisibilityState }[]
}

// --- Game State ---

export class GameState {
  tick: number = 0
  phase: MatchPhase = 'active'
  mapConfig: MapConfig
  terrain: Tile[][]
  units: Map<string, Unit> = new Map()
  buildings: Map<string, Building> = new Map()
  resources: Map<string, ResourceNode> = new Map()
  players: PlayerState[] = []
  commandQueue: GameCommand[] = []
  visibility: VisibilityState[][][] = [] // per-player visibility
  winnerSlot: number | null = null

  private previousState: {
    unitPositions: Map<string, Position>
    unitStatuses: Map<string, UnitStatus>
    unitIds: Set<string>
    buildingIds: Set<string>
    playerEnergies: number[]
  } | null = null

  constructor(mapConfig: MapConfig) {
    this.mapConfig = mapConfig

    // Initialize terrain from map config or default to grass
    if (
      mapConfig.terrain &&
      typeof mapConfig.terrain === 'object' &&
      Array.isArray(mapConfig.terrain)
    ) {
      this.terrain = mapConfig.terrain as Tile[][]
    } else {
      this.terrain = Array.from({ length: mapConfig.height }, () =>
        Array.from({ length: mapConfig.width }, () => ({ type: 'grass' as TileType, elevation: 1 }))
      )
    }

    // Initialize resource nodes
    for (const node of mapConfig.resourceNodes) {
      const id = crypto.randomUUID()
      this.resources.set(id, {
        id,
        x: node.x,
        y: node.y,
        remaining: node.energy,
      })
    }

    // Initialize players and bases
    const playerCount = mapConfig.startPositions.length
    for (let slot = 0; slot < playerCount; slot++) {
      const pos = mapConfig.startPositions[slot]
      const baseId = crypto.randomUUID()

      this.buildings.set(baseId, {
        id: baseId,
        playerSlot: slot,
        buildingType: 'base',
        x: pos.x,
        y: pos.y,
        hp: BASE_STATS.hp,
        maxHp: BASE_STATS.hp,
      })

      this.players.push({
        slot,
        energy: STARTING_ENERGY,
        baseId,
      })

      // Initialize per-player visibility
      this.visibility.push(
        Array.from({ length: mapConfig.height }, () =>
          Array.from({ length: mapConfig.width }, () => 'unseen' as VisibilityState)
        )
      )
    }
  }

  snapshotForDiff() {
    this.previousState = {
      unitPositions: new Map([...this.units].map(([id, u]) => [id, { x: u.x, y: u.y }])),
      unitStatuses: new Map([...this.units].map(([id, u]) => [id, u.status])),
      unitIds: new Set(this.units.keys()),
      buildingIds: new Set(this.buildings.keys()),
      playerEnergies: this.players.map((p) => p.energy),
    }
  }

  computeDiff(): StateDiff {
    const diff: StateDiff = {
      tick: this.tick,
      unitsMoved: [],
      unitsSpawned: [],
      unitsKilled: [],
      buildingsBuilt: [],
      buildingsDestroyed: [],
      combatEvents: [],
      resourcesChanged: [],
      visibilityUpdates: [],
    }

    if (!this.previousState) return diff

    // Units moved
    for (const [id, unit] of this.units) {
      const prev = this.previousState.unitPositions.get(id)
      const prevStatus = this.previousState.unitStatuses.get(id)
      if (prev && (prev.x !== unit.x || prev.y !== unit.y || prevStatus !== unit.status)) {
        diff.unitsMoved.push({ id, x: unit.x, y: unit.y, status: unit.status })
      }
    }

    // Units spawned
    for (const [id, unit] of this.units) {
      if (!this.previousState.unitIds.has(id)) {
        diff.unitsSpawned.push({ ...unit })
      }
    }

    // Units killed
    for (const id of this.previousState.unitIds) {
      if (!this.units.has(id)) {
        diff.unitsKilled.push(id)
      }
    }

    // Buildings destroyed
    for (const id of this.previousState.buildingIds) {
      if (!this.buildings.has(id)) {
        diff.buildingsDestroyed.push(id)
      }
    }

    // Resources changed
    for (let i = 0; i < this.players.length; i++) {
      if (this.players[i].energy !== this.previousState.playerEnergies[i]) {
        diff.resourcesChanged.push([i, this.players[i].energy])
      }
    }

    return diff
  }

  isBlocked(x: number, y: number): boolean {
    if (x < 0 || y < 0 || x >= this.mapConfig.width || y >= this.mapConfig.height) return true
    const tile = this.terrain[y]?.[x]
    if (tile && TILE_MOVEMENT_COST[tile.type] === Infinity) return true
    for (const building of this.buildings.values()) {
      if (building.x === x && building.y === y && building.buildingType === 'wall') return true
    }
    return false
  }

  /** Get movement cost for a tile. Returns Infinity if impassable. */
  getMovementCost(x: number, y: number): number {
    if (x < 0 || y < 0 || x >= this.mapConfig.width || y >= this.mapConfig.height) return Infinity
    const tile = this.terrain[y]?.[x]
    if (!tile) return 1.0
    return TILE_MOVEMENT_COST[tile.type]
  }

  getPlayerBase(slot: number): Building | undefined {
    const player = this.players[slot]
    return player ? this.buildings.get(player.baseId) : undefined
  }

  getVisibleState(playerSlot: number): Record<string, unknown> {
    const vis = this.visibility[playerSlot]
    const visibleUnits: Unit[] = []
    const visibleBuildings: Building[] = []
    const visibleResources: ResourceNode[] = []

    for (const unit of this.units.values()) {
      if (unit.playerSlot === playerSlot || vis[unit.y]?.[unit.x] === 'visible') {
        visibleUnits.push({ ...unit })
      }
    }

    for (const building of this.buildings.values()) {
      if (building.playerSlot === playerSlot || vis[building.y]?.[building.x] === 'visible') {
        visibleBuildings.push({ ...building })
      }
    }

    for (const resource of this.resources.values()) {
      if (vis[resource.y]?.[resource.x] === 'visible') {
        visibleResources.push({ ...resource })
      }
    }

    return {
      tick: this.tick,
      energy: this.players[playerSlot].energy,
      units: visibleUnits,
      buildings: visibleBuildings,
      resources: visibleResources,
      mapWidth: this.mapConfig.width,
      mapHeight: this.mapConfig.height,
    }
  }
}
