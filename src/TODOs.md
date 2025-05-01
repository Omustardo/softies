Bugs and necessary improvements:
* avoid stretching of polygons outside of their bounds, probably due to division by zero or similar?
* when creatures reach the mouse, they jiggle a lot. Ideally they would stay steady.
* prevent self-intersection so the segments move smoothly
* improve the outline and skin of creatures.

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
* Falling sand, but it's attracted to the skeleton/rigging of a creature. Could look very cool? Ideally it would somewhat lock into place once it surrounded the skeleton, becoming sort of fleshy.
