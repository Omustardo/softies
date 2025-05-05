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

## Day 1 (2025-04-27)

I installed Cursor, signed up for its a free trial, and got started.

In the first evening I spent 4 solid hours. Getting a basic demo (drawing a circle) set up and compiling for both desktop and wasm was very quick and easy. Adding constraints that allowed a circle to follow another circle, and then a sort of chain-link of circles was also quite quick. Even adding points to each side of the circles (for anchor points in the future) was relatively quick. The main things I spent time on were:
* Widget ID conflicts. Something about buttons with the same ID was causing `egui` to display an error. Cursor simply couldn't figure it out on its own after many attempts. I ended up getting Cursor to figure it out by asking it to remove UI elements, and then I'd run the program and see if the issue occurred. It narrowed it down to being in a certain panel in the UI, and from there it was able to change its approach. https://github.com/Omustardo/softies/commit/5dfe2788f2cab97fe372b597d87b3730dcc700a8
* Drawing out of bounds. When drawing the skin on creatures, it was meant to fill in each segment of the creature. It was drawing a big thing from the head all the way to the tail. I was probably misleading it with my prompts here because I was telling it that the skin was wrong, when technically it was the infill between the points defined by the skin. Still, it took ages for me to get tired of letting it do its attempts. Once I made a new prompt and specifically told it that infill was the issue, it fixed it. https://github.com/Omustardo/softies/commit/0a59a4e8ae28f74fdbce050de517ed055f17d63a
* Latency / Jank. Cursor kept going in the wrong direction when debugging this. It tried to use `puffin` which is a Rust profiler, but it couldn't get it working. I steered it away and it tried debug logging, but the logging all seemed fine. At this point I realized there was a web search option. That gave it some other things to try. It still couldn't get it though. What worked was asking it whether it could be a rendering issue rather than CPU usage and it figured out that there was a missing `request_repaint()`: https://github.com/Omustardo/softies/commit/ccdbebd944a66030393d007f0c729bb575e8e332

This used up almost all of my 150 "premium model queries" for Cursor's trial period! Based on Cursor costing $20/mo and giving 500 of these, that's roughly $6 spent on API costs. Not nothing, but inexpensive for what I got out of it.

The experience was a mix of initially being very pleased at things working, and then significant frustration with Cursor being unable to fix issues despite my varied prompts with different ways to approach it. In the end it made a segmented worm / snake that moved toward the mouse cursor. There were buttons to add and remove add segments, and a tree view to modify segments of the current creature. Each node was connected to one or two other nodes, and they had infill between them as a sort of skin. It worked well on both web and desktop. Definitely more than I could've done on my own in four hours unless it were in a language and with a library that I was familiar with. The downsides is that I don't know the codebase very well since I accepted changes without much review. If the LLM digs itself into a hole it'll be difficult for me to help it out. This early stage of the project is also the most likely for the LLM to do well with since everything easily fits into its context window and it's mostly boilerplate. I suspect things will only get harder for it.

## Day 2 (2025-04-30)

Another 4 hour session. I paid $20 for a month of Cursor. It's understandable, but also a shame that the free trial is so few queries. It's also a shame it doesn't support local LLMs. To avoid blowing through my Cursor quota in a couple sessions, I used Google's AI Studio and Claude alongside Cursor today. I think AI Studio (Gemini) is the best LLM for coding right now, but Claude has integration with Github so it's easier to select the relevant files as context, and it's still good. I also have a year-long subscription for Claude from a few months ago when it was the strongest model, so I may as well use it now.

I tried to make creature movement more realistic by adding constraints. For example, a snake shouldn't be able to bend entirely back on itself. Each pair of segments needs to have a limit to how far it can bend. I started off asking Cursor to implement constraints between segments which it failed at horribly. It didn't add constraints properly, and also caused the snake to ball up and have jittery movement. I realized that it was getting stuck after a couple queries and so I searched for an existing physics library. Giving it a link to that (Rapier.rs) seemed to be a better path forward, though it repeatedly hit compilation errors. It was sort of annoying to have to keep giving it the error messages and having it fix things. Perhaps there's a way to let it automatically apply "safe" changes.

I continued to work on this issue for the entire evening. For whatever reason, this was extremely hard for it. I used lots of strategies: like finding demo code for it, having it write its own minimal demo, having it search the web for example code, having it generate and use tests, use fuzz testing to figure out good physics parameters. I had to provide it with https://rapier.rs/docs/user_guides/rust/common_mistakes as it kept using the wrong scale (e.g. 1 pixel = 1 meter, so it barely moves at all).

In the end I finally got it working by going back to my initial approach of creating a basic Rapier demo and working off of that. The experience this evening was much less pleasant than the first session. It felt like far less got done, and I had to try a lot harder to make progress. I ended up using ~80 queries on Cursor (about half of what I used last session). Still, it did get something working in the end, so I can't complain too much. LLMs are still feeling magical, even if working with them can be frustrating and doesn't feel like I'm bettering myself.


## Day 3 (2025-05-02)

Another 4 hours. Roughly the same experience as the last session. I was paying less attention to the process this time compared to prior sessions. Lots of entering queries and not paying attention while it wrote out the responses. Unsurprisingly it had a lot of difficulty.

I am continuing to use egui for the UI and Rapier as the physics engine. Cursor had suggested trying out Bevy (a game engine) in response to some prompts about improving physics. I gave it a try and the code worked on Desktop, but try as I might I couldn't get it to display anything in WebAssembly. I spent a while trying to get this to work but gave up and went back to egui+Rapier. I think I was generally expecting too much from it and giving it low effort descriptions of what I wanted it to do. Didn't make much progress.

I used about 100 Cursor queries.

## Day 4 (2025-05-04)

