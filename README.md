# softies

Soft body animations based on https://www.youtube.com/@argonautcode:
* https://www.youtube.com/watch?v=qlfh_rv6khY
* https://www.youtube.com/watch?v=GXh0Vxg7AnQ

Generated almost entirely by vibe coding using Cursor.

# Development process

I want to learn Rust and I've been meaning to try out vibe coding. I had seen argonautcode's videos a while back and wanted to try out implementing them. I've used LLMs ([Claude](https://claude.ai/new) and [Gemini](https://aistudio.google.com/prompts/new_chat)) in their web UIs to generate standalone scripts and other code. I have used LLMs in IDEs with very limited success, and usually abject failure and wasting of time. This is a brand new project in a space that has been solidly covered before, so I thought it might be a good fit.

In the first evening I spent 4 solid hours. Getting a basic demo (drawing a circle) set up and compiling for both desktop and wasm was very quick and easy. Adding constraints that allowed a circle to follow another circle, and then a sort of chain-link of circles was also quite quick. Even adding points to each side of the circles (for anchor points in the future) was relatively quick. The main things I spent time on were:
* Widget ID conflicts. Something about buttons with the same ID was causing `egui` to display an error. Cursor simply couldn't figure it out on its own after many attempts. I ended up getting Cursor to figure it out by asking it to remove UI elements, and then I'd run the program and see if the issue occurred. It narrowed it down to being in a certain panel in the UI, and from there it was able to change its approach. https://github.com/Omustardo/softies/commit/5dfe2788f2cab97fe372b597d87b3730dcc700a8
* Drawing out of bounds. When drawing the skin on creatures, it was meant to fill in each segment of the creature. It was drawing a big thing from the head all the way to the tail. I was probably misleading it with my prompts here because I was telling it that the skin was wrong, when technically it was the infill between the points defined by the skin. Still, it took ages for me to get tired of letting it do its attempts. Once I made a new prompt and specifically told it that infill was the issue, it fixed it. https://github.com/Omustardo/softies/commit/0a59a4e8ae28f74fdbce050de517ed055f17d63a
* Latency / Jank. Cursor kept going in the wrong direction when debugging this. It tried to use `puffin` which is a Rust profiler, but it couldn't get it working. I steered it away and it tried debug logging, but the logging all seemed fine. At this point I realized there was a web search option. That gave it some other things to try. It still couldn't get it though. What worked was asking it whether it could be a rendering issue rather than CPU usage and it figured out that there was a missing `request_repaint()`: https://github.com/Omustardo/softies/commit/ccdbebd944a66030393d007f0c729bb575e8e332
