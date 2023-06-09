# Rolf3D

My own implementation of **Wolfenstein 3D**, using Rust :)

**TODO: Put some more documentation here !!!!**

## Useful Links to original code

- [ThreeDRefresh](https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_DRAW.C#L1331)
- [DrawPlayerWeapon](https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_DRAW.C#L1191)

## TODO

### In progress / next TODOs:

- FULL Status bar
- Correct Automap (same as ECWolf)
- Elevator / correctly move between floors
  - IDEA: keep the same LiveMap instance between floors
    - select episode + floor (default = 0) at LiveMap construction
    - load map method
  - also: secret elevator + return to correct floor afterwards
    - STORE return floor when the secret elevator is used!
  - keep guns, score, lives, health and ammo between floors (but lose the keys)
  - correctly display the current floor in the status bar !!

### Code Cleanup TODOs

- REFACTORING:
  - generalize InputManager + move it to SDL code + put SDL code in one source
  - move 3D rendering to a separate mod, to separate rendering from game world modeling?
    - might be problematic - both rendering and modeling are tied to livemap !!
    - maybe go EVEN FURTHER - 3 mods!!
      - livemap (the data, VERY public)
      - worldmodel (manipulates the data)
      - render3d (renders the map)
- FIX known BUGS:
  - BUG: PWall speed is NOT OK + PWalls _only move 2 tiles_
    - see [original code for PWalls](https://github.com/id-Software/wolf3d/blob/05167784ef009d0d0daefe8d012b027f39dc8541/WOLFSRC/WL_ACT1.C#L727)
  - BUG: wall collision is sometimes broken at wall corners :(
  - BUG: sprites + player weapons are still not aligned properly => TWEAK IT !!
    - see [SimpleScaleShape](https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_SCALE.C#L604)
  - make sure the SPEEDS for doors, push walls are CORRECT
  - FIX: my world is flipped vertically => the unit circle is also flipped :(
    => try to unflip it !!!
- PROPER CODE DOCS
- Fix TODOs in code + code cleanup !!
- Put some more documentation here !!!!

### Future TODOs

- Movement part 2:
  - mouse horizontal turn
  - mouse buttons
  - running (Shift / CapsLock)
- Map investigations:
  - What is the meaning of each THING word, in the maps ?
  - make 2 TABLEs below:
    - all Tile types + their Texture IDXs
    - all Thing types + their Sprite IDXs
- FIXES:
  - [Timed pallete shift on damage + pickup](https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_PLAY.C#L1131)
  - transition between floors:
    - flip switch in elevator
    - show summary screen
    - transition to the new floor
  - Fix TODOs in code + code cleanup !!
- Actors!
  - static 4 now + hurt-able + they hurt back on shooting + show some indication of the damage they did
- Sounds preparation:
  - add notifications (+ temporary messages) for ALL situations where a sound would be played
- Sounds ?!?
  - How to play them via SDL ??
  - How to extract them from AUDIOHED / AUDIOT / VSWAP ??
- MENUs
  - Main Menu + Title Screen
  - Pause Menu
  - Options (see EcWolf ?!)
- Gameplay
  - correct ceiling color
  - key handling (e.g. Tab = Automap)
- (IS THIS NEEDED ?) identify PIC indexes based on game type (WL1, WL6, SOD, SDM)
  - seems to matter only if I want to reproduce EXACTLY the original game
- Enemy AI
  - "enemy direction" things ?!?
- Enemy and sound propagation
  - propagate within area
  - when door is open/opening/closing => the 2 areas become connected !!
- High Scores
- Full(er) Game:
  - SOD:
    - correct pic indexes
    - full title pic, with palette
  - Shift-Alt-Backspace -> see [Cheats for Wolf3D](https://steamcommunity.com/sharedfiles/filedetails/?id=150838966)
- FULL game - like the originals :/
  - Save games
  - Sound (!?!)
- Remove Strum dependency

### Done

- Basic painting via ScreenBuffer + draw text
- Load assets from files:
  - load palette from GAMEPAL.OBJ + hardcode it + display it
  - load maps and sketch them (just colored rectangles, for now)
  - load graphics assets: VSWAP (flats and sprites) + VGAGRAPH (fonts and pics)
- 3D View / Raycaster - walls and doors (very basic, no clipping)
- Basic movement through the 3D world: move, turn, strife
- Player Bounds = collision detection with walls, wall sliding
- Open doors, with timed animation and timeout-to-close
- BUGFIX: doors MUST NOT CLOSE while an actor is inside the door cell
- Push Walls, with timed movement
- VERY BASIC status bar
- Correct pic indexes (incomplete 4 SoD)
- Blocking vs unblocking decoration sprites
- Collectibles: treasures, weapons, ammo, health etc
- In-game timed messages
- draw held weapon in 3D

## INVESTIGATION NOTES

### Tiles / textures

| Tile code   | Texture index | Notes                             |
| ----------- | ------------- | --------------------------------- |
| 1 ... 89    | 2n-2 / 2n-1   | Solid walls (1)                   |
| -> 13       | 24, 25        | - "solid" elevator doors (2)      |
| -> 16       | 30, 31        | - day / night sky                 |
| -> 21       | 40 / 41       | - elevator inside                 |
| -> 21       | 43            | - flipped elevator switch         |
| 90 ... 101  | (see below )  | Doors (3)                         |
| -> 90, 91   | 99, 98        | - normal door                     |
| -> 92, 93   | 105, 104      | - locked door (gold key)          |
| -> 94, 95   | 105, 104      | - locked door (blue key)          |
| -> 96..99   | 105, 104      | - locked door (UNUSED)            |
| -> 100, 101 | 103, 102      | - elevator door                   |
| 102 ... 105 | -             | UNUSED                            |
| 106         | -             | "Ambush" marker (4)               |
| 107         | -             | Area tile (5) / Alt. elevator (6) |
| 108 ...     | -             | Area code (empty space)           |

- (1) each solid wall (with tile code `n`) has 2 textures:
  - `2*n - 2` (even) is a _light_ version, for the N/S faces of the wall
  - `2*n - 1` (even) is a _dark_ version, for the E/W faces of the wall
- (2) tile 13 is used as "solid" elevator door, at the entrance of each floor
- (3) _even_ door codes are for _vertical doors_ (with _dark_ textures), and _odd_ codes are for _horizontal doors_ (with _light_ textures)
- (4) AMBUSHTILE (106) => special meaning for enemies (also, it is a non-solid tile)
- (5) AREATILE (107) => tiles which are >= 107 are empty (walkable) space + represent an _area code_
  - _area_ = a room where sound propagates - e.g. enemies are alerted in case of gunshot
  - NOTE: when a door is NOT closed, it connects the 2 areas (so sound propagates between them) !!!
- (6) ALTELEVATORTILE (also 107) => if the player stands on this tile when activating an elevator => it goes to the secret level !!
- door textures:
  - 98, 99 = regular door
  - 100, 101 = door edges
  - 102, 103 = elevator doors
  - 104, 105 = locked door
- see:
  - [SetupGameLevel - solid tiles etc](https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_GAME.C#L665)
  - [InitDoorList - door tiles](https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_GAME.C#L688)

### Things / sprites

TODO - this needs more improvements and investigations !!

| Thing code | Sprite index | Notes                                 |
| ---------- | ------------ | ------------------------------------- |
| 0          | -            | Empty (no thing)                      |
| 19..22     | -            | player start position                 |
| 23..74     | n - 21       | static items (1)                      |
| 90..97     | -            | directioning markers                  |
| 98         | -            | "push wall" marker                    |
| 108-213    | ???          | various enemies and their orientation |

- for actors, there are 4 codes, for the 4 possible spawn orientations (TODO find the orientation for each code ?!)
- 19 ... 22 = player start position + initial orientation
- 23 ... 74 = various [static items](https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_ACT1.C) - decorations, collectibles etc
  - sprite idx = THING - 21
  - GOLD key thing code = 43 (sprite = 22)
  - SILVER key thing code = 44 (sprite = 23)
- (??) 90 ... 97 = directioning for patroling enemies (TODO verify this !!)
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
