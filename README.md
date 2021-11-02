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
 - Collision system
 - Shooting
 - Minimap
 



# Misc ideas

level gen - multiple levels, have some big boys that seldom change direction and then spawn a bunch of other ones

gameplay -- could make clear time be a factor

could have dark levels, visibility cone, scary shit
progress levels by finding exits, so you kind of opt into sewers or whatever its your own fault. like labyrinth in poe
have tresury rooms etc

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

# Misc issues

theres the odd visual artifact
eg a black tear
or entities kinda warp, maybe needs some interpolation
the autocomplete in this project is actually fucked, I wonder if its glams fault