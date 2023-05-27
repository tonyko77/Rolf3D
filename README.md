# Rolf3D

My own implementation of **Wolfenstein 3D**, using Rust :)

**TODO: Put some more documentation here !!!!**

## TODO - implementation steps:

- REFACTORINGS - add new classes:

  - InputManager (input.rs) - handles keyboard & mouse, knows if key/mousebtn is pressed, set key/btn timings
  - LiveMapSimulator (livemap.rs) - simulates the game world -> player, doors, actors, AI, timings etc
  - AutomapRenderer (automap.rs) - renders the automap using the LiveMapSimulator
  - ThreeDRenderer (render3d.rs) - renders the 3D world using the LiveMapSimulator

- Map investigations:

  - What is the meaning of each WALL and THING word, in the map arrays ?!?
  - Is plane #3 really used/needed? and is it really empty for ALL maps in WL1/WL6/SOD ??

- Automap:

  - display walls/doors etc using actual graphics
  - display things using actual graphics

- Movement on the Automap:

  - turn and strife
  - mouse horizontal turn
  - mouse buttons
  - collision detection (with walls)

- 3D View / Raycaster

  - floor + correct ceiling color
  - walls and doors
  - sprites (actors, decorations)
  - movement through the 3D world

- Put some more documentation here !!!!

- Gameplay

  - key handling (e.g. Tab = Automap)
  - open doors !!

- (IS THIS NEEDED ?) identify PIC indexes based on game type (WL1, WL6, SOD, SDM)

  - seems to matter only if I want to reproduce EXACTLY the original game

- Enemy AI

  - shoot/knife enemies

- (Almost) Full Game:
  - NO sound :/
  - Menu system
- Save games
- Sound (?!?)

## DONE

- basic painting via ScreenBuffer
- load palette from GAMEPAL.OBJ + hardcode it + display it
- load maps and sketch them (just colored rectangles, for now)
- load graphics assets: VSWAP (flats and sprites) + VGAGRAPH (fonts and pics)
- test-paint gfx assets - use <Tab> to switch between automap and Gfx
- draw text

## INVESTIGATION NOTES

- Map format (tiles, things etc):

  - tiles:
    - if tile == AMBUSHTILE (106) => special meaning (probably: enemy in ambush mode)
      - also, it's actually a non-solid tile :)
    - if tile < AREATILE (107) => solid wall
    - if tile in [90..101] => door, vertical if even, lock = (tile - 90|91)/2
      - 100/101 are elevator doors (looks like only 100 is ever used)
    - if tile >= 108 => empty cell
      - NOT SURE: seems like different values are used here, to distinguish btw rooms
  - tile textures:
    - for a SOLID tile with value x => texture code is: (x-1)\*2 + 0|1
      - there are 2 textures per tile code: LIGHT (for N|S) and DARK (for E|W)
    - for a DOOR with x in [90..99] => texture code is: (x+8)
    - for the ELEVATOR door, the texture code is: 24
    - TODO - confirm all this in the WOLF3D code !!!
  - how to detect solid wall - https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_GAME.C#L665

    - if (tile < AREATILE) => solid wall !!
    - if (tile >= 90 && tile <= 101) => door, vertical if even, lock = (tile - 90|91)/2
      - https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_GAME.C#L688
    - some interesting constants -> https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_DEF.H#L61

      ```
      #define PUSHABLETILE     98
      #define EXITTILE         99   // at end of castle
      #define AREATILE         107  // first of NUMAREAS floor tiles
      #define NUMAREAS         37
      #define ELEVATORTILE     21
      #define AMBUSHTILE       106
      #define ALTELEVATORTILE  107
      ```

    - things, player star => see ScanInfoPlane
      -> https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_GAME.C#L221
