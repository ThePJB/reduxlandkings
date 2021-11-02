goal of this? I reckon make a SDL style renderer and reimplement rustlandkings

* geometry sent from cpu - done
* render draw rect
* render present

(oh or camera or inputs)

yeah do rustlandkings itll be easier, figure out inputs

--------

OKOKOKOK whats the matter.
no colours. why?

only draws first triangle, why?
buffers look correct to me

drawing wrong
alignment wrong?


======

ok lets do rustlandkings mk2

ECS
 - generational index? for fast rendering we want a nice contiguous array

ColourMesh component:
 - draw all entities with 1 draw call
 * I suppose more normal/OO would be a mesh object
   * transform per object, draw call per object?

AABB component:
 - for collision

PlayerController component:
 - duh

AIController component:
 - duh

 honestly maybe sdl style rendering not so stupid

 I will have renderer,
 I will have ECS with hashmaps

================

App: Renderer, input schemas, etc

    v input commands v rendering v

Game: Look, subjective player stuff etc

    v contains v entity commands v

Level: entities and simulation stuff, no specific player
    
    ^ entity commands for AI

=====================

TODO:
 - Fix resolution
 - Entities
 - Collision
 - Shooting
 - Minimap
 
 - add entities, hook up player
 - sort out aspect ratio

 How im gonna do view matrix?
basically just translate to position of player + 0.2 * look
make sure its not some weird recursive shit with picking
easier w/o mats?


level gen - multiple levels, have some big boys that seldom change direction and then spawn a bunch of other ones


gameplay -- could make clear time be a factor