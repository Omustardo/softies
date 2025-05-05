Goal: create a 2D aquarium ecosystem with soft-bodied creatures.

Bugs and necessary improvements:
* Refine Plankton energy recovery (make it dependent on being near the surface).
* Start implementing the Snail.
* Start implementing the Fish.
* Work on creature sensing/interactions (eating).

Movment:
* Add currents that push everything. Probably need to add weights to each segment to simulate this well?
* Constrain everything to within the viewing area (no going out of bounds).
* Add a state machine to each creature (e.g. wander, hunt, eat, flee, etc)

Creatures:
* plankton / amoeabae. single celled creatures. ensure that latency is fine with many of them.
* seaweed. some long and wavy, some branching, some clusters that float around.
* eels that live in the sand
* fish. needs a fin on top and on the sides, with nice flowing animation. Need a way for it to turn to the sides.
* remora (sticks to larger creatures)
* lizard / salamander. Something with legs.

Stretch / longer term:
* Predators and prey. Replication. Balancing of an ecosystem (add light source for plants / plankton).
* Decouple the view region with the movement region, and allow zooming in and out (with the scrollwheel on desktop, and two fingers if accessing the wasm version on mobile).
* How hard would it be to make these 3d?
* improve the outline and skin of creatures.
* Falling sand, but it's attracted to the skeleton/rigging of a creature. Could look very cool? Ideally it would somewhat lock into place once it surrounded the skeleton, becoming sort of fleshy.




The UI elements that add segments to creatures aren't working. Let's just get rid of those buttons and replace them with a "creature examiner". When a creature is clicked, we should switch to "examine" mode, which keeps the creature always in view (maybe pinned in place, or our view is locked to it)

When clicked, there should be a display on a side of the screen with information about the creature, and an option to make the camera follow the center of the creature. "Center" should be one of the segments of the creature.
