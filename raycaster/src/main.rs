use macroquad::prelude as mq;
use rayon::prelude::*;

const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 512;

const MAP_WIDTH: u32 = 8;
const MAP_HEIGHT: u32 = 8;
const TILE_SIZE: u32 = 64;

const NUM_RAYS: u32 = 512;
const FOV: f32 = std::f32::consts::PI / 2.0;

struct Player {
    pos: mq::Vec2,
    direction: mq::Vec2,
    angle: f32, // in radians
}
impl Player {
    fn new(pos: mq::Vec2) -> Self {
        Self {
            pos,
            angle: 0.0,
            direction: mq::Vec2::new(1.0, 0.0),
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
    fn keyboard(&mut self, delta: f32) {
        if mq::is_key_down(mq::KeyCode::Left) {
            self.angle -= 3.0 * delta;
            if self.angle < 0.0 {
                self.angle += 2.0 * std::f32::consts::PI;
            }
            self.direction = mq::Vec2::new(self.angle.cos(), self.angle.sin());
        }
        if mq::is_key_down(mq::KeyCode::Right) {
            self.angle += 3.0 * delta;
            if self.angle > 2.0 * std::f32::consts::PI {
                self.angle -= 2.0 * std::f32::consts::PI;
            }
            self.direction = mq::Vec2::new(self.angle.cos(), self.angle.sin());
        }

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
    fn cast_rays(&self, map: &[u8]) -> Vec<Option<(mq::Vec2, bool)>> {
        // let mut rays = Vec::new();
        // for i in 0..NUM_RAYS {
        //     let angle = self.angle - FOV / 2.0 + FOV * i as f32 / NUM_RAYS as f32;
        //     let ray = Ray {
        //         pos: self.pos,
        //         direction: mq::Vec2::new(angle.cos(), angle.sin()),
        //     };
        //     rays.push(ray.cast_ray(map));
        // }
        // rays
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

struct Ray {
    pos: mq::Vec2,
    direction: mq::Vec2,
}
impl Ray {
    fn cast_ray(&self, map: &[u8]) -> Option<(mq::Vec2, bool)> {
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
                    // Is fisheye solved? - No?
                    // let angle_between = ray_dir.angle_between(center_dir);
                    // let angle_between = std::f32::consts::PI / 2.0 - angle_between.abs();
                    // let distance = distance * angle_between.cos();
                    return Some((self.pos + (ray_dir * distance * TILE_SIZE as f32), x_move));
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

    loop {
        if mq::is_key_down(mq::KeyCode::Escape) {
            break;
        }

        mq::clear_background(mq::GRAY);
        mq::draw_rectangle(
            WINDOW_WIDTH as f32 / 2.0,
            WINDOW_HEIGHT as f32 / 2.0,
            WINDOW_WIDTH as f32 / 2.0,
            WINDOW_HEIGHT as f32 / 2.0,
            mq::DARKGRAY,
        );

        let delta = mq::get_frame_time(); // seconds

        draw_map(&map);

        player.keyboard(delta);
        player.draw();
        let ray_touches = player.cast_rays(&map);

        // flatten removes the Option
        for (ray_touch, x_move) in ray_touches.iter().flatten() {
            let color = if *x_move {
                mq::Color::new(0.9, 0.2, 0.2, 1.0)
            } else {
                mq::Color::new(0.6, 0.1, 0.1, 1.0)
            };
            mq::draw_line(
                player.pos.x,
                player.pos.y,
                ray_touch.x,
                ray_touch.y,
                3.0,
                color,
            );

            let dist = (player.pos - *ray_touch).length();
            if dist < 0.1 {
                continue;
            }
            let angle = (player.pos - *ray_touch).angle_between(player.direction);

            let projection_dist = (TILE_SIZE as f32 / 2.0) / (FOV / 2.0).tan();
            let z = dist * angle.cos();
            let h = (WINDOW_HEIGHT as f32 * projection_dist) / z;

            let projection_pos = 0.5 * angle.tan() / (FOV / 2.0).tan();
            let x =
                (WINDOW_WIDTH as f32 / 2.0) * (0.5 - projection_pos) + (WINDOW_WIDTH as f32 / 2.0);

            mq::draw_rectangle(x, (WINDOW_HEIGHT as f32 - h) / 2.0, 1.0, h, color);
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
