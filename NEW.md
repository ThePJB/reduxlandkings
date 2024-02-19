Magic is cooler. Lets do the spellbook page turner thing. 

Find spell pages. Put in ur spellbook. Frantically flip through.

Maybe crafting.

Multiple casting schemes: uses mana. uses health: blood magic. generates heat - more powerful at critical levels but also flirting with death

fog of war, clairvoyance, utility spells etc.
some kind of plane grammar, how to find ultimate goal, transcendence
find dead wizards and scraps of their journals with information
about discovery and pursuing transcendence at any cost
divine comedy

beams, wards, delays, aoe, self kill, healing, time manip, etc, buff now i do aoe. projectile defence., bouncing. juicy particles.
seeing through fog of war
seeing directions to stuff
seeing secret sugar
start modifying portals, creating them, why not. have to start conducting your own search through the multiverse, using rules like, going to the dark version of each instance, find the true ending

bit of an existance metaphor with observer selection, here I am in random portals.
bit of less is more organizational overhead
structure, rules of portals like what type of instance they lead to
e.g. icy, find icy spells
structure that a pro could exploit for a certain build

maybe procedural sigils on spell pages that say what it is.

look dont be tempted to do procedural, let different schools of magic have a unique feel.
certain firetrucked old rift sections have certain types of magic.
reckless discovery, maybe you just go through random portals too. This is my life now. One shot at greatness.

diabloi gui

phase jump, same level, different sugar, different interpretation. Could absolutely generate with different constraints. It could be puzzly to work out how to go. Get on a different frequency

# Level Gen

have stages of sections of certain parameters. open, closed, elemental affiliation, types of enemies, treasure rooms. Use techniques from that roguelike level generation video:
* CA
    0 neigh: wall, 1-4: empty, 5+ wall
    region based CA rules
    weigh 8conn less than 4conn, kernels. kernel space
    just '< 5' looks continenty
* random walk of varying parameters
* pathfinding to cull unreachable, shortest path to constrain
* room place / bsp rooms
* room + DLA - eroded
* prefab interfacing, masking etc
* DLA: start with a seed, shoot particles from a random pos in a random dir and if they hit it, floor
    or central attractor, only shoot in middle
    or haev a path as seed
* voronoi
* noise
* wfc / tiling

* djikstra stuff
    cull unreachable
    choose start/end points
    start point, end point, hot path
    rooms: essential vs non essential
    also invoke ordering on rooms

fat drunkard
partial symmetry

# First things first

darkness, fog of war why not
next up djikstra path, start and end portal
distance to path cull
then multiple ends, points of interest hide treasure etc.
bsp rooms combine with ca spaces

multi cavern where it varies

can totally find a start and end point on each one and then chain em, say for dla or whatever or making sure they meet 





# Components of level generation
* A primitive: logical matrix
    operations: stitching together at certain points, masking, etc distance culling etc,
* Player placement. eg furthest point from start
* Enemy placement. not within radius of player

ok mr wolverson. I think I wanna do a more functional style. MapFragment monoid

better have hazards, room prefabs, placements |= in some rooms. conv |= not set. circle fragment. scale up
noise. think masking.