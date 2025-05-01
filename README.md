# softies

Soft body animations based on https://www.youtube.com/@argonautcode:
* https://www.youtube.com/watch?v=qlfh_rv6khY
* https://www.youtube.com/watch?v=GXh0Vxg7AnQ

Generated almost entirely by vibe coding using Cursor.

# Development process

I want to learn Rust and I've been meaning to try out vibe coding. I had seen argonautcode's videos about soft body animation and wanted to try out implementing them. I've used LLMs ([Claude](https://claude.ai/new) and [Google's AI Studio (Gemini)](https://aistudio.google.com/prompts/new_chat)) in their web UIs to generate standalone scripts and other code which has generally gone very well. I have used LLMs in IDEs too, but with very limited success, and usually abject failure and overall spending more time on LLM issues than it would've taken me. This is a brand new project in a space that has been solidly covered before by other work, so I thought it might be a good fit for trying out vibe coding.

Goals:
* Create soft body animations
* Use Rust
* Target both Desktop and WebAssembly
* Create some cool creatures. Maybe an aquarium for my website and/or a screensaver?

## Day 1 (2024-04-27)

I installed Cursor, signed up for its a free trial, and got started.

In the first evening I spent 4 solid hours. Getting a basic demo (drawing a circle) set up and compiling for both desktop and wasm was very quick and easy. Adding constraints that allowed a circle to follow another circle, and then a sort of chain-link of circles was also quite quick. Even adding points to each side of the circles (for anchor points in the future) was relatively quick. The main things I spent time on were:
* Widget ID conflicts. Something about buttons with the same ID was causing `egui` to display an error. Cursor simply couldn't figure it out on its own after many attempts. I ended up getting Cursor to figure it out by asking it to remove UI elements, and then I'd run the program and see if the issue occurred. It narrowed it down to being in a certain panel in the UI, and from there it was able to change its approach. https://github.com/Omustardo/softies/commit/5dfe2788f2cab97fe372b597d87b3730dcc700a8
* Drawing out of bounds. When drawing the skin on creatures, it was meant to fill in each segment of the creature. It was drawing a big thing from the head all the way to the tail. I was probably misleading it with my prompts here because I was telling it that the skin was wrong, when technically it was the infill between the points defined by the skin. Still, it took ages for me to get tired of letting it do its attempts. Once I made a new prompt and specifically told it that infill was the issue, it fixed it. https://github.com/Omustardo/softies/commit/0a59a4e8ae28f74fdbce050de517ed055f17d63a
* Latency / Jank. Cursor kept going in the wrong direction when debugging this. It tried to use `puffin` which is a Rust profiler, but it couldn't get it working. I steered it away and it tried debug logging, but the logging all seemed fine. At this point I realized there was a web search option. That gave it some other things to try. It still couldn't get it though. What worked was asking it whether it could be a rendering issue rather than CPU usage and it figured out that there was a missing `request_repaint()`: https://github.com/Omustardo/softies/commit/ccdbebd944a66030393d007f0c729bb575e8e332

This used up almost all of my 150 "premium model queries" for Cursor's trial period! Based on Cursor costing $20/mo and giving 500 of these, that's roughly $6 spent on API costs. Not nothing, but inexpensive for what I got out of it.

The experience was a mix of initially being very pleased at things working, and then significant frustration with Cursor being unable to fix issues despite my varied prompts with different ways to approach it. In the end it made a segmented worm / snake that moved toward the mouse cursor. There were buttons to add and remove add segments, and a tree view to modify segments of the current creature. Each node was connected to one or two other nodes, and they had infill between them as a sort of skin. It worked well on both web and desktop. Definitely more than I could've done on my own in four hours unless it were in a language and with a library that I was familiar with. The downsides is that I don't know the codebase very well since I accepted changes without much review. If the LLM digs itself into a hole it'll be difficult for me to help it out. This early stage of the project is also the most likely for the LLM to do well with since everything easily fits into its context window and it's mostly boilerplate. I suspect things will only get harder for it.
