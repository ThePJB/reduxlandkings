# Reduxlandkings
Top down shooter roguelike (nuclear throne inspired)
using SDL style renderer built on OpenGL (no C dependencies yay!!!!!)


# Architecture
App: Renderer, input schemas, etc

    v input commands v rendering v

Game: Look, subjective player stuff etc

    v contains v entity commands v

Level: entities and simulation stuff, no specific player
    
    ^ entity commands for AI


# TODO
 - bullet collisions etc
 - health
 - Minimap - probably need to fuck off camera matrix. or use a different one. get to the bottom of it
 
## Guns
OK so what is a gun?
 * gets updated every frame 
 * either is told to shoot or it isnt 
 * either shoots or it doesnt
   * edge cases: burst fire where it shoots a minimum number of rounds

 * State kept to decide whether to shoot
   * when last shot
   * how much ammo
   * other previous information about shooting (how many of burst etc it can get complicated)
   * heat or something if its ammoless

 * other parameters

 * can you have it be a linear combination so you can have a gun combining system? / procedural guns
   * beneficial characteristics: damage, multishot, bullet velocity, bouncing, AoE, lightning arcs
   * negative characteristics: cooldown, burst cooldown, ammo use, forced firing, chance to jam, spread, recoil, screen shake, overheat, reload procedure, charge up
   * ammo types for more variety as well?

 * maybe rust type system can make this pretty easy e.g. just have a bunch of maybe components
 
 * dude crafting and combining guns would be really fun, kinda PoEy which is always good
 * why not do PoE multiverse stuff too

 there was that gun fifo idea too. robot makes sense if you do it in esoteric ways. maybe you are a killbot and you hate all this organic life
 could add and send to back for crafting
 maybe a mutate and send to back or + and send to back whatever
 right click eat for hp or something, maybe it takes a sec

you should be a 3 legged robot and game be full of robot jokes
what was that idea I had before about all your guns use a stack, sure that might be fun as well

procedural stuff that actually has gameplay implications is a good doctrine



# Misc ideas

level gen - multiple levels, have some big boys that seldom change direction and then spawn a bunch of other ones

gameplay -- could make clear time be a factor

could have dark levels, visibility cone, scary shit
progress levels by finding exits, so you kind of opt into sewers or whatever its your own fault. like labyrinth in poe
have tresury rooms etc

## Gun ideas
 - pistol, boring
 - rifle, cool
 - spray and pray smg



## Enemies
 - A fatty that spawns little guys
    - could spawn up to a limit or be static or roam the world, get a ramp up in difficulty, encourage swiftness
 - fast melee guys
 - unpredictable spray and pray shooter
 - fat tanky guy
 - retalliator is pretty good
 - squad tactics
 - uses cover
 - has shield friends that tank
 - sprinter, gets puffed
 - explodes on death (but friendly fire?)
 - suicide guys
 - lazy dudes that dont follow you
 - bullet dodgers
 - guy who splits in two on death
 - guy who pieces of him fly off and spawn smaller guys
 - dudes who only run at you when you look at them
 - dudes who run away when you look at them, backstabbers. shotty

## Walkers
 - 3x3
 - spawns exit
 - short erratic diggy
 - spawns short erratic diggy
 - long distance, spawns cluster


## Mechanics
 - different guns should come and go, make you adapt and change it up
 - eating guns for health
 - eating guns for $$$

 battery mechanic? rewarding clear speed
 maybe instead of progress on clear spawn an exit
 good if it could signpost the kind of area
 random sort of levels
 pretty CA eg for grass or whatever
 give enemies more of a brain for object permanence + wandering
 also can base behaviour on their guns burst cooldown

# Misc issues
(Medium) Why autocomplete etc jank in this project, is it glams fault? maybe its better to just remember shit anyway
(Minor) black tear artifact
(Minor) collision system hitch on walls in -X and -Y direction

they sometimes freeze if player isnt moving, looks pretty unnatural lol
  select an entity to see its thoughts
  raycasting is buggy


  probably just have set transform in renderer
  draw some lines and shit, get to the bottom of why raycasting is being a fucc
  click on enemies to see their brain work
  yea soundz g



ok fixing transforms etc is getting there. Where should it live? Probably in game.
game just needs to know if aspect ratio changes.
game should be able to do screen to world and world to screen (world to screen uses renderer)
also remove size spaghetti

reimplement look



ok time to implement minimap and hppppp

a lot of AI stuff could be implicit random procedural eg tendency to run or not as a fn of time