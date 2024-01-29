# 3D-Raycaster

[Live Demo](https://lelserslasers.itch.io/3d-raycaster): https://lelserslasers.itch.io/3d-raycaster

Wolfenstein 3D like raycaster using Macroquad in Rust running on the web

## Features

- Digital differential analyser (DDA) raycaster
	- Like Wolfenstein 3D
- Textured walls
	- With multiple textures
- Minimap
- Ability to look up and down
- Fog effect
- Wall collisions
- This was mostly a proof of concept
	- The math was actually pretty fun to figure out and I have used the reverse of it (screen positions to angles, distances, etc) for robotics 

## Showcase

![Showcase 1](./Showcase/Screenshot1.png)
![Showcase 2](./Showcase/Screenshot2.png)

## Controls

- WASD to move
- Arrow keys to turn/look around
- R to replay "loading" animation
- Mouse to movement to look around (must click inside window to grab mouse)
- Tab to release mouse grab


## Copyright stuff

- Textures are from ID Software's Wolfenstein 3D

## Bugs

- Speed
	- Only CPU bound?
- Mouse grab on Windows??
- Mouse grab on web?
