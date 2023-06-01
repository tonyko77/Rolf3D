# Rolf3D

My own implementation of **Wolfenstein 3D**, using Rust :)

**TODO: Put some more documentation here !!!!**

## TODO

### In progress / next TODOs:

- (NEXT) Push Walls
- REFACTORING:
  - move 3D rendering to a separate mod, to separate rendering from game world modeling?
    - might be problematic - both rendering and modeling are tied to livemap !!
    - maybe go EVEN FURTHER - 3 mods!!
      - livemap (the data, VERY public)
      - worldmodel (manipulates the data)
      - render3d (renders the map)
- PROPER CODE DOCS + code cleanup !!
- BUG: gets very laggy/locked sometimes
  - seems to be from painting sprites (NOT sure)
  - => DOES this still REPRODUCE ??

### Future TODOs

- Things part 2:
  - blocking vs unblocking decorations
  - collectibles (treasures, weapons, ammo, health etc)
  - Actors! (static 4 now)
- Movement part 2:
  - mouse horizontal turn
  - mouse buttons
  - running (Shift / CapsLock)
- Map investigations:
  - What is the meaning of each WALL and THING word, in the map arrays ?!?
  - Is plane #3 really used/needed? and is it really empty for ALL maps in WL1/WL6/SOD ??
- Automap:
  - display walls/doors etc using actual graphics
  - display things using actual graphics
- Put some more documentation here !!!!
- Gameplay
  - floor + correct ceiling color
  - key handling (e.g. Tab = Automap)
  - open doors !!
- (IS THIS NEEDED ?) identify PIC indexes based on game type (WL1, WL6, SOD, SDM)
  - seems to matter only if I want to reproduce EXACTLY the original game
- Enemy AI
  - shoot/knife enemies
- Full(er) Game:
  - ok with NO sound :/
  - Menu system + title pic
  - Pause menu
- FULL game
  - Save games
  - Sound (!?!)

### Done

- basic painting via ScreenBuffer
- load palette from GAMEPAL.OBJ + hardcode it + display it
- load maps and sketch them (just colored rectangles, for now)
- load graphics assets: VSWAP (flats and sprites) + VGAGRAPH (fonts and pics)
- test-paint gfx assets - use <Tab> to switch between automap and Gfx
- draw text
- 3D View / Raycaster - walls and doors (very basic, no clipping)
- basic movement through the 3D world: move, turn, strife
- Player Bounds = collision detection with walls
  - - improved algorithm for collision detection, to enable wall sliding :)
- Open doors, with timed animation and timeout-to-close
- Sprites - only decoration sprites, for now (+ no blocking decorations yet)
- BUGFIX: doors MUST NOT CLOSE while an actor is inside the door cell

## INVESTIGATION NOTES

### Map format (tiles, things etc):

**Special tile codes:**

- 1 ... 89 => solid walls
  - 21 = end level elevator
- 90 ... 101 => doors, vertical if even, lock = (tile - 90|91)/2
  - 90/91 are regular doors
  - 92/93 = TODO: locked doors ?!?
  - 100/101 are elevator doors:
    - seems that 100 is only used for the exit elevator (it is always a "vertical" door)
    - ... while 101 is for the "entrance" to each level >= 2 (cannot be opened)
    - also, 101 HAS NO EDGES (it's at the same level as the neighbouring walls)
- AMBUSHTILE (106) => special meaning for enemies (also, it is a non-solid tile)
- AREATILE (107) => tiles which are >= 107 are empty (walkable) space + represent an _area code_ (probably a room, where e.g. enemies are alerted in case of gunshot)
- see:
  - [SetupGameLevel - solid tiles etc](https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_GAME.C#L665)
  - [InitDoorList - door tiles](https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_GAME.C#L688)

**Tile textures:**

- if SOLID with x in 1..89 => texture code is: 2x-2 / 2x-1
  - there are 2 textures per tile code: LIGHT (for N|S) and DARK (for E|W)
- for a DOOR with x in [90..99] => texture code is: (x+8)
- 100, exit elevator door (dark) => 25
- 101, entrance elevator door (light) => 24
- door edge texture (for any type of door) => 100

**Thing codes:**

- 0 = empty tile (no thing)
- 19 ... 22 = player start position + initial orientation
- 23 ... 74 = various [static items](https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_ACT1.C) - decorations, collectibles etc
- 98 = pushable walls, a.k.a. _secrets_
- various enemies, depending on level:
  - `en_guard`:
    - `SpawnStand` => 180-183 (hard), 144-147 (med+hard), 108-111 (always)
    - `SpawnPatrol` => 184-187 (hard), 148-151 (med+hard), 112-115 (always)
    - "dead" sprite: 124
  - `en_officer`:
    - `SpawnStand` => 188-191 (hard), 152-155 (med+hard), 116-119 (always)
    - `SpawnPatrol` => 192-195 (hard), 156-159 (med+hard), 120-123 (always)
    - "dead" sprite: ????
  - `en_ss`:
    - `SpawnStand` => 198-201 (hard), 162-165 (med+hard), 126-129 (always)
    - `SpawnPatrol` => 202-205 (hard), 166-169 (med+hard), 130-133 (always)
    - "dead" sprite: ????
  - `en_dog`
    - `SpawnStand` => 206-209 (hard), 170-173 (med+hard), 134-137 (always)
    - `SpawnPatrol` => 210-213 (hard), 174-177 (med+hard), 138-141 (always)
    - "dead" sprite: ????
  - Wolfenstein/Spear Bosses etc - see sources
- [orientation "suffixes"](https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_DEF.H#L123)
  - 0 = North
  - 1 = East
  - 2 = South
  - 3 = West
- see [ScanInfoPlane](https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_GAME.C#L221)

[Some interesting constants](https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_DEF.H#L61)

```
#define PUSHABLETILE     98
#define EXITTILE         99   // at end of castle
#define AREATILE         107  // first of NUMAREAS floor tiles
#define NUMAREAS         37
#define ELEVATORTILE     21
#define AMBUSHTILE       106
#define ALTELEVATORTILE  107
```
