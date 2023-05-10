# 3D-Raycaster

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

## Showcase

![Showcase 1](./Showcase/Screenshot1.png)
![Showcase 2](./Showcase/Screenshot2.png)

## Controls

- WASD to move
- Arrow keys to turn/look around
- R to replay "loading" animation
- Mouse to movement to look around (must click inside window to grab mouse)
- Esc to release mouse grab


## Copyright stuff

- Textures are from ID Software's Wolfenstein 3D

## Bugs

- Window resizing
	- Non-web - enforce 2:1 aspect ratio
- Speed
	- Only CPU bound?
- Mouse grab on Windows??
- Mouse grab on web?
