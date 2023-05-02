use macroquad::prelude as mq;
use rayon::prelude::*;

const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 512;

const MAP_WIDTH: u32 = 8;
const MAP_HEIGHT: u32 = 8;
const TILE_SIZE: u32 = 64;

const NUM_RAYS: u32 = 512;
const FOV: f32 = std::f32::consts::PI / 2.0;

const MOUSE_SENSITIVITY: f32 = 0.001;

const VIEW_DISTANCE: f32 = 7.0 * TILE_SIZE as f32;

const BACKGROUND_COLOR: mq::Color = mq::Color::new(73.0 / 255.0, 1.0, 1.0, 1.0);
const GROUND_COLOR: mq::Color = mq::Color::new(36.0 / 255.0, 219.0 / 255.0, 0.0, 1.0);
const WALL_COLOR_LIGHT: mq::Color = mq::Color::new(0.6, 0.6, 0.6, 1.0);
const WALL_COLOR_DARK: mq::Color = mq::Color::new(0.55, 0.55, 0.55, 1.0);

struct Player {
    pos: mq::Vec2,
    direction: mq::Vec2,
    angle: f32,          // in radians
    angle_vertical: f32, // in radians

    last_mouse_pos: mq::Vec2,
}
impl Player {
    fn new(pos: mq::Vec2) -> Self {
        Self {
            pos,
            angle: 0.0,
            angle_vertical: 0.0,
            direction: mq::Vec2::new(1.0, 0.0),
            last_mouse_pos: mq::mouse_position().into(),
        }
    }
    fn draw(&self) {
        mq::draw_circle(self.pos.x, self.pos.y, 8.0, mq::YELLOW);
        mq::draw_line(
            self.pos.x,
            self.pos.y,
            self.pos.x + self.angle.cos() * 20.0,
            self.pos.y + self.angle.sin() * 20.0,
            3.0,
            mq::YELLOW,
        );
    }
    fn input(&mut self, delta: f32, mouse_grabbed: bool) {
        if mq::is_key_down(mq::KeyCode::Left) {
            self.angle -= 3.0 * delta;
        }
        if mq::is_key_down(mq::KeyCode::Right) {
            self.angle += 3.0 * delta;
        }

        // 2.1 = slightly less than 90 degrees
        if mq::is_key_down(mq::KeyCode::Up) {
            self.angle_vertical += 3.0 * delta;
        }
        if mq::is_key_down(mq::KeyCode::Down) {
            self.angle_vertical -= 3.0 * delta;
        }

        let mouse_position: mq::Vec2 = mq::mouse_position().into();
        let mouse_delta = mouse_position - self.last_mouse_pos;
        self.last_mouse_pos = mouse_position;
        
        if mouse_grabbed {
            self.angle += mouse_delta.x * MOUSE_SENSITIVITY;
            self.angle_vertical -= mouse_delta.y * MOUSE_SENSITIVITY;
        }

        if self.angle < 0.0 {
            self.angle += 2.0 * std::f32::consts::PI;
        } else if self.angle > 2.0 * std::f32::consts::PI {
            self.angle -= 2.0 * std::f32::consts::PI;
        }
        if self.angle_vertical > std::f32::consts::PI / 2.1 {
            self.angle_vertical = std::f32::consts::PI / 2.1;
        } else if self.angle_vertical < -std::f32::consts::PI / 2.1 {
            self.angle_vertical = -std::f32::consts::PI / 2.1;
        }

        self.direction = mq::Vec2::new(self.angle.cos(), self.angle.sin());

        let mut move_vec = mq::Vec2::new(0.0, 0.0);
        if mq::is_key_down(mq::KeyCode::W) {
            move_vec += self.direction;
        }
        if mq::is_key_down(mq::KeyCode::S) {
            move_vec -= self.direction;
        }
        if mq::is_key_down(mq::KeyCode::A) {
            move_vec += mq::Vec2::new(self.direction.y, -self.direction.x);
        }
        if mq::is_key_down(mq::KeyCode::D) {
            move_vec -= mq::Vec2::new(self.direction.y, -self.direction.x);
        }

        if move_vec.length() > 0.0 {
            move_vec = move_vec.normalize();
            self.pos += move_vec * 100.0 * delta;
            if self.pos.x < 0.0 {
                self.pos.x = 0.0;
            } else if self.pos.x > MAP_WIDTH as f32 * TILE_SIZE as f32 {
                self.pos.x = MAP_WIDTH as f32 * TILE_SIZE as f32;
            }

            if self.pos.y < 0.0 {
                self.pos.y = 0.0;
            } else if self.pos.y > MAP_HEIGHT as f32 * TILE_SIZE as f32 {
                self.pos.y = MAP_HEIGHT as f32 * TILE_SIZE as f32;
            }
        }
    }
    fn cast_rays(&self, map: &[u8]) -> Vec<Option<RayHit>> {
        (0..NUM_RAYS)
            .into_par_iter()
            .map(|i| {
                let angle = self.angle - FOV / 2.0 + FOV * i as f32 / NUM_RAYS as f32;
                let ray = Ray {
                    pos: self.pos,
                    direction: mq::Vec2::new(angle.cos(), angle.sin()),
                };
                ray.cast_ray(map)
            })
            .collect()
    }
}

struct RayHit {
    pos: mq::Vec2,
    world_distance: f32,
    x_move: bool,
    wall_coord: f32, // 0-1.0 as x
}
struct Ray {
    pos: mq::Vec2,
    direction: mq::Vec2,
}
impl Ray {
    fn cast_ray(&self, map: &[u8]) -> Option<RayHit> {
        // DDA algorithm

        let x = self.pos.x / TILE_SIZE as f32; // (0.0, 8.0)
        let y = self.pos.y / TILE_SIZE as f32; // (0.0, 8.0)
        let ray_start = mq::Vec2::new(x, y);

        let ray_dir = self.direction.normalize();

        let ray_unit_step_size = mq::Vec2::new(
            (1.0 + (ray_dir.y / ray_dir.x).powi(2)).sqrt(),
            (1.0 + (ray_dir.x / ray_dir.y).powi(2)).sqrt(),
        );
        let mut map_check = ray_start.floor();
        let mut ray_length_1d = mq::Vec2::ZERO;
        let mut step = mq::Vec2::ZERO;

        if ray_dir.x < 0.0 {
            step.x = -1.0;
            ray_length_1d.x = (x - map_check.x) * ray_unit_step_size.x;
        } else {
            step.x = 1.0;
            ray_length_1d.x = (map_check.x + 1.0 - x) * ray_unit_step_size.x;
        }

        if ray_dir.y < 0.0 {
            step.y = -1.0;
            ray_length_1d.y = (y - map_check.y) * ray_unit_step_size.y;
        } else {
            step.y = 1.0;
            ray_length_1d.y = (map_check.y + 1.0 - y) * ray_unit_step_size.y;
        }

        let max_distance = 100.0;
        let mut distance = 0.0;
        let mut x_move;
        while distance < max_distance {
            if ray_length_1d.x < ray_length_1d.y {
                map_check.x += step.x;
                distance = ray_length_1d.x;
                ray_length_1d.x += ray_unit_step_size.x;
                x_move = true;
            } else {
                map_check.y += step.y;
                distance = ray_length_1d.y;
                ray_length_1d.y += ray_unit_step_size.y;
                x_move = false;
            }

            if map_check.x >= 0.0
                && map_check.x < MAP_WIDTH as f32
                && map_check.y >= 0.0
                && map_check.y < MAP_HEIGHT as f32
            {
                let map_index = (map_check.y * MAP_WIDTH as f32 + map_check.x) as usize;
                if map[map_index] == 1 {
                    let pos = self.pos + (ray_dir * distance * TILE_SIZE as f32);

                    let map_pos = pos / TILE_SIZE as f32;
                    let wall_pos = map_pos - map_pos.floor();
                    let wall_coord = if x_move {
                        wall_pos.y
                    } else {
                        wall_pos.x
                    };
                    
                    return Some(RayHit {
                        pos,
                        world_distance: distance * TILE_SIZE as f32,
                        x_move,
                        wall_coord,
                    });
                }
            }
        }

        None
    }
}

fn draw_map(map: &[u8]) {
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let wall = map[(y * MAP_WIDTH + x) as usize];
            if wall == 1 {
                mq::draw_rectangle(
                    x as f32 * TILE_SIZE as f32 + 1.0,
                    y as f32 * TILE_SIZE as f32 + 1.0,
                    TILE_SIZE as f32 - 2.0,
                    TILE_SIZE as f32 - 2.0,
                    mq::WHITE,
                );
            } else {
                mq::draw_rectangle(
                    x as f32 * TILE_SIZE as f32 + 1.0,
                    y as f32 * TILE_SIZE as f32 + 1.0,
                    TILE_SIZE as f32 - 2.0,
                    TILE_SIZE as f32 - 2.0,
                    mq::BLACK,
                );
            }
        }
    }
}

fn window_conf() -> mq::Conf {
    mq::Conf {
        window_title: "3D Raycaster".to_owned(),
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut player = Player::new(mq::Vec2::new(
        WINDOW_WIDTH as f32 / 4.0,
        WINDOW_HEIGHT as f32 / 2.0,
    ));

    let mut mouse_grapped = false;
    mq::set_cursor_grab(mouse_grapped);
    mq::show_mouse(!mouse_grapped);

    #[rustfmt::skip]
    let map = [
        1, 0, 0, 0, 0, 0, 0, 1,
        1, 0, 0, 0, 0, 0, 0, 1,
        1, 0, 0, 0, 0, 0, 0, 1,
        1, 0, 0, 1, 1, 0, 0, 1,
        1, 0, 0, 0, 0, 0, 0, 1,
        1, 0, 0, 1, 0, 1, 0, 1,
        1, 0, 0, 0, 0, 0, 0, 1,
        1, 1, 1, 1, 1, 1, 1, 1,
    ];
    // let map = [
    //     0, 0, 0, 0, 0, 0, 0, 0,
    //     0, 0, 0, 0, 0, 0, 0, 0,
    //     0, 0, 0, 0, 0, 0, 0, 0,
    //     0, 0, 0, 0, 0, 0, 0, 0,
    //     0, 0, 0, 0, 0, 0, 0, 0,
    //     0, 0, 0, 0, 0, 0, 1, 0,
    //     0, 0, 0, 0, 0, 0, 0, 0,
    //     0, 0, 0, 0, 0, 0, 0, 0,
    // ];

    let wall_texture = mq::load_texture("resources/stolenWall.png").await.unwrap();

    loop {
        if mq::is_key_down(mq::KeyCode::Escape) {
            break;
        }
        if mq::is_key_pressed(mq::KeyCode::Tab) || mq::is_mouse_button_pressed(mq::MouseButton::Left) {
            mouse_grapped = !mouse_grapped;
            mq::set_cursor_grab(mouse_grapped);
            mq::show_mouse(!mouse_grapped);
        }

        mq::clear_background(BACKGROUND_COLOR);

        let floor_level =
            (WINDOW_HEIGHT as f32 / 2.0) * (1.0 + player.angle_vertical.tan() / (FOV / 2.0).tan());

        mq::draw_rectangle(
            WINDOW_WIDTH as f32 / 2.0,
            floor_level,
            WINDOW_WIDTH as f32 / 2.0,
            WINDOW_HEIGHT as f32 - floor_level,
            GROUND_COLOR,
        );

        let delta = mq::get_frame_time(); // seconds

        draw_map(&map);

        player.input(delta, mouse_grapped);
        player.draw();
        let ray_touches = player.cast_rays(&map);

        // flatten removes the Option
        for ray_hit in ray_touches.iter().flatten() {
            let color = if ray_hit.x_move {
                WALL_COLOR_LIGHT
            } else {
                WALL_COLOR_DARK
            };
            mq::draw_line(
                player.pos.x,
                player.pos.y,
                ray_hit.pos.x,
                ray_hit.pos.y,
                3.0,
                color,
            );

            if ray_hit.world_distance < 0.1 {
                continue;
            }
            let angle = (player.pos - ray_hit.pos).angle_between(player.direction);

            let projection_dist = (TILE_SIZE as f32 / 2.0) / (FOV / 2.0).tan();

            let z = ray_hit.world_distance * angle.cos();
            let h = (WINDOW_HEIGHT as f32 * projection_dist) / z;

            let projection_pos = 0.5 * angle.tan() / (FOV / 2.0).tan();
            let x =
                (WINDOW_WIDTH as f32 / 2.0) * (0.5 - projection_pos) + (WINDOW_WIDTH as f32 / 2.0);

            mq::draw_texture_ex(wall_texture, x - 1.0, floor_level - (h / 2.0), mq::WHITE, mq::DrawTextureParams {
                dest_size: Some(mq::Vec2::new(2.0, h)),
                source: Some(mq::Rect::new(
                    ray_hit.wall_coord * wall_texture.width(),
                    0.0,
                    2.0,
                    wall_texture.height(),
                )),
                ..Default::default()
            });
            // mq::draw_rectangle(x - 1.0, floor_level - (h / 2.0), 2.0, h, color);

            let fog_brightness = (2.0 * ray_hit.world_distance / VIEW_DISTANCE - 1.0).max(0.0);
            let fog_color = mq::Color::new(
                BACKGROUND_COLOR.r,
                BACKGROUND_COLOR.g,
                BACKGROUND_COLOR.b,
                fog_brightness,
            );

            mq::draw_rectangle(x - 1.0, floor_level - (h / 2.0), 2.0, h, fog_color);
        }

        mq::draw_text(
            format!("FPS: {}", mq::get_fps()).as_str(),
            5.,
            15.,
            20.,
            mq::BLUE,
        );
        mq::draw_text(
            format!("DELTA: {:.2} ms", delta * 1000.0).as_str(),
            5.,
            30.,
            20.,
            mq::BLUE,
        );

        mq::next_frame().await
    }
}
