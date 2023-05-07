use macroquad::prelude as mq;

const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 512;

const MAP_WIDTH: u32 = 8;
const MAP_HEIGHT: u32 = 8;
const TILE_SIZE: u32 = 64;

const NUM_RAYS: u32 = 512;
const FOV: f32 = std::f32::consts::PI / 2.0;

const MOUSE_SENSITIVITY: f32 = 0.001;

const VIEW_DISTANCE: f32 = 7.0 * TILE_SIZE as f32;

const NUM_TEXTURES: i32 = 3;

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
    fn touching_wall(&mut self, move_vec: mq::Vec2, delta: f32, map: &[u8]) {
        let move_x = move_vec.x * 100.0 * delta;
        let move_y = move_vec.y * 100.0 * delta;

        self.pos.x += move_x;
        let map_x = (self.pos.x / TILE_SIZE as f32) as usize;
        let map_y = (self.pos.y / TILE_SIZE as f32) as usize;
        let map_index = map_y * MAP_WIDTH as usize + map_x;

        if map[map_index] != 0 {
            self.pos.x -= move_x;
        }

        self.pos.y += move_y;
        let map_x = (self.pos.x / TILE_SIZE as f32) as usize;
        let map_y = (self.pos.y / TILE_SIZE as f32) as usize;
        let map_index = map_y * MAP_WIDTH as usize + map_x;

        if map[map_index] != 0 {
            self.pos.y -= move_y;
        }
    }
    fn input(&mut self, delta: f32, mouse_grabbed: bool, map: &[u8]) {
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
            self.touching_wall(move_vec, delta, map);

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
    fn cast_rays(&self, map: &[u8]) -> Vec<(Ray, Option<RayHit>)> {
        let rotation_matrix = mq::Mat2::from_angle(self.angle);
        (0..NUM_RAYS)
            .map(|i| {
                // let angle = self.angle - FOV / 2.0 + FOV * i as f32 / NUM_RAYS as f32;
                let unrotated_direction =
                    mq::Vec2::new(1.0, (i as f32 / NUM_RAYS as f32 - 0.5) * FOV);
                let direction = rotation_matrix * unrotated_direction;
                let ray = Ray::new(self.pos, direction);
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
    wall_type: u8,
}
#[derive(Clone, Copy)]
struct Ray {
    pos: mq::Vec2,
    angle: f32,
    direction: mq::Vec2,
}
impl Ray {
    fn new(pos: mq::Vec2, direction: mq::Vec2) -> Self {
        Self {
            pos,
            angle: direction.y.atan2(direction.x),
            direction,
        }
    }
    fn cast_ray(&self, map: &[u8]) -> (Ray, Option<RayHit>) {
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
                let wall_type = map[map_index];
                if wall_type != 0 {
                    // 0 = no wall
                    let pos = self.pos + (ray_dir * distance * TILE_SIZE as f32);

                    let map_pos = pos / TILE_SIZE as f32;
                    let wall_pos = map_pos - map_pos.floor();
                    let wall_coord = if x_move { wall_pos.y } else { wall_pos.x };

                    return (
                        *self,
                        Some(RayHit {
                            pos,
                            world_distance: distance * TILE_SIZE as f32,
                            x_move,
                            wall_coord,
                            wall_type,
                        }),
                    );
                }
            }
        }

        (*self, None)
    }
}

trait Lerp {
    fn lerp(self, other: Self, t: f32) -> Self;
}
impl Lerp for mq::Color {
    fn lerp(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        mq::Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }
}

fn draw_map(map: &[u8]) {
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let wall = map[(y * MAP_WIDTH + x) as usize];
            let color = match wall {
                1 => mq::BLUE,
                2 => mq::RED,
                3 => mq::GREEN,
                _ => mq::BLACK,
            };
            mq::draw_rectangle(
                x as f32 * TILE_SIZE as f32 + 1.0,
                y as f32 * TILE_SIZE as f32 + 1.0,
                TILE_SIZE as f32 - 2.0,
                TILE_SIZE as f32 - 2.0,
                color,
            );
        }
    }
}

struct VerticalLine {
    x: i32,
    y0: i32,
    y1: i32,
}
impl VerticalLine {
    fn new(x: i32, y0: i32, y1: i32) -> Self {
        Self { x, y0, y1 }
    }
}
fn vertical_line(line: VerticalLine, output_image: &mut mq::Image, color: mq::Color) {
    let x = line.x.clamp(0, output_image.width() as i32 - 1) as u32;
    let y0 = line.y0.clamp(0, output_image.height() as i32 - 1) as u32;
    let y1 = line.y1.clamp(0, output_image.height() as i32 - 1) as u32;

    for y in y0..y1 {
        output_image.set_pixel(x, y, color);
    }
}

fn vertical_textured_line_with_fog(
    wall_line: VerticalLine,
    output_image: &mut mq::Image,
    texture: &mq::Image,
    texture_line: VerticalLine,
    fog_brightness: f32,
) {
    let draw_x = wall_line.x.clamp(0, output_image.width() as i32 - 1) as u32;
    let draw_y0 = wall_line.y0.clamp(0, output_image.height() as i32 - 1) as u32;
    let draw_y1 = wall_line.y1.clamp(0, output_image.height() as i32 - 1) as u32;

    let texture_x = texture_line.x.clamp(0, texture.width() as i32 - 1) as u32;

    let h = wall_line.y1 - wall_line.y0;
    let texture_h = texture_line.y1 - texture_line.y0;

    for y in draw_y0..draw_y1 {
        let h_ratio = texture_h as f32 / h as f32;
        let h_diff = y as i32 - wall_line.y0;
        let texture_y = (h_diff as f32 * h_ratio) as u32 + texture_line.y0 as u32;

        let color = texture.get_pixel(texture_x, texture_y);
        let color_with_fog = color.lerp(BACKGROUND_COLOR, fog_brightness);
        output_image.set_pixel(draw_x, y, color_with_fog);
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
        WINDOW_WIDTH as f32 / 4.0 + TILE_SIZE as f32 / 2.0,
        WINDOW_HEIGHT as f32 / 2.0 + TILE_SIZE as f32 / 2.0,
    ));

    let mut mouse_grapped = false;
    mq::set_cursor_grab(mouse_grapped);
    mq::show_mouse(!mouse_grapped);

    #[rustfmt::skip]
    let map = [
        1, 0, 0, 0, 0, 0, 0, 1,
        1, 0, 0, 0, 0, 0, 0, 2,
        2, 0, 0, 0, 0, 0, 0, 3,
        2, 0, 0, 1, 3, 0, 0, 3,
        3, 0, 0, 0, 0, 0, 0, 2,
        3, 0, 0, 3, 0, 2, 0, 1,
        1, 0, 0, 0, 0, 0, 0, 1,
        1, 3, 3, 3, 2, 1, 2, 1,
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

    let wall_image = mq::Image::from_file_with_format(
        include_bytes!("../resources/WolfensteinTextures.png"),
        Some(mq::ImageFormat::Png),
    );

    let mut output_image =
        mq::Image::gen_image_color(WINDOW_WIDTH as u16 / 2, WINDOW_HEIGHT as u16, mq::WHITE);
    let output_texture = mq::Texture2D::from_image(&output_image);

    loop {
        if mq::is_key_pressed(mq::KeyCode::Escape)
            || mq::is_mouse_button_pressed(mq::MouseButton::Left)
        {
            mouse_grapped = !mouse_grapped;
            mq::set_cursor_grab(mouse_grapped);
            mq::show_mouse(!mouse_grapped);

            // TODO: BUG: doesn't work on windows
            // println!("Mouse grapped: {}", mouse_grapped);
        }

        // mq::clear_background(BACKGROUND_COLOR);

        let floor_level =
            (WINDOW_HEIGHT as f32 / 2.0) * (1.0 + player.angle_vertical.tan() / (FOV / 2.0).tan());

        // mq::draw_rectangle(
        //     WINDOW_WIDTH as f32 / 2.0,
        //     floor_level,
        //     WINDOW_WIDTH as f32 / 2.0,
        //     WINDOW_HEIGHT as f32 - floor_level,
        //     GROUND_COLOR,
        // );

        let delta = mq::get_frame_time(); // seconds

        draw_map(&map);

        player.input(delta, mouse_grapped, &map);
        player.draw();
        let ray_touches = player.cast_rays(&map);

        for (i, ray_touch) in ray_touches.iter().enumerate() {
            let ray = &ray_touch.0;
            let ray_hit = &ray_touch.1;

            let x = i as i32;

            if let Some(ray_hit) = ray_hit {
                let angle_between = player.angle - ray.angle;
                let z = ray_hit.world_distance * angle_between.cos();

                let projection_dist = (TILE_SIZE as f32 / 2.0) / (FOV / 2.0).tan();

                let h = (WINDOW_HEIGHT as f32 * projection_dist) / z;
                let y0 = floor_level - (h / 2.0);
                let y1 = y0 + h;

                let y0 = y0.round() as i32;
                let y1 = y1.round() as i32;

                let texture_x = (ray_hit.wall_coord * wall_image.width() as f32).round() as i32;
                let texture_y0 =
                    (wall_image.height() as i32 / NUM_TEXTURES) * (ray_hit.wall_type as i32 - 1);
                let texture_y1 = texture_y0 + wall_image.height() as i32 / NUM_TEXTURES;

                let sky = VerticalLine::new(x, 0, y0);
                vertical_line(sky, &mut output_image, BACKGROUND_COLOR);

                let fog_brightness = (2.0 * ray_hit.world_distance / VIEW_DISTANCE - 1.0).max(0.0);
                // let wall_color = mq::WHITE.lerp(BACKGROUND_COLOR, fog_brightness);
                // vertical_line(x, y0, y1, &mut output_image, wall_color);

                let wall_line = VerticalLine::new(x, y0, y1);
                let texture_line = VerticalLine::new(texture_x, texture_y0, texture_y1);
                vertical_textured_line_with_fog(
                    wall_line,
                    &mut output_image,
                    &wall_image,
                    texture_line,
                    fog_brightness,
                );

                let floor = VerticalLine::new(x, y1, WINDOW_HEIGHT as i32);
                vertical_line(floor, &mut output_image, GROUND_COLOR);

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
            } else {
                let floor_y = floor_level.round() as i32;

                let sky = VerticalLine::new(x, 0, floor_y);
                vertical_line(sky, &mut output_image, BACKGROUND_COLOR);

                let floor = VerticalLine::new(x, floor_y, WINDOW_HEIGHT as i32);
                vertical_line(floor, &mut output_image, GROUND_COLOR);
            }
        }
        output_texture.update(&output_image);
        mq::draw_texture(output_texture, WINDOW_WIDTH as f32 / 2.0, 0.0, mq::WHITE);

        // let mut previous_x = WINDOW_WIDTH as f32 / 2.0;

        // for i in 0..ray_touches.len() {
        //     let ray = &ray_touches[i].0;
        //     let ray_hit = &ray_touches[i].1;

        //     let angle_between = player.angle - ray.angle;
        //     let projection_pos = 0.5 * angle_between.tan() / (FOV / 2.0).tan();
        //     let x =
        //         (WINDOW_WIDTH as f32 / 2.0) * (0.5 - projection_pos) + (WINDOW_WIDTH as f32 / 2.0);

        //     if x < previous_x {
        //         continue;
        //     }
        //     let w = if i == ray_touches.len() - 1 {
        //         WINDOW_WIDTH as f32 - previous_x
        //     } else {
        //         x - previous_x
        //     };

        //     if let Some(ray_hit) = ray_hit {

        //         let color = if ray_hit.x_move {
        //             WALL_COLOR_LIGHT
        //         } else {
        //             WALL_COLOR_DARK
        //         };
        //         mq::draw_line(
        //             player.pos.x,
        //             player.pos.y,
        //             ray_hit.pos.x,
        //             ray_hit.pos.y,
        //             3.0,
        //             color,
        //         );
        //         // let angle = (player.pos - ray_hit.pos).angle_between(player.direction);
        //         let projection_dist = (TILE_SIZE as f32 / 2.0) / (FOV / 2.0).tan();

        //         let z = ray_hit.world_distance * angle_between.cos();
        //         let h = (WINDOW_HEIGHT as f32 * projection_dist) / z;

        //         mq::draw_texture_ex(
        //             wall_texture,
        //             previous_x,
        //             floor_level - (h / 2.0),
        //             mq::WHITE,
        //             mq::DrawTextureParams {
        //                 dest_size: Some(mq::Vec2::new(w, h)),
        //                 source: Some(mq::Rect::new(
        //                     ray_hit.wall_coord * wall_texture.width() - w / 2.0,
        //                     (wall_texture.height() / NUM_TEXTURES)
        //                         * (ray_hit.wall_type as f32 - 1.0),
        //                     w,
        //                     wall_texture.height() / NUM_TEXTURES,
        //                 )),
        //                 // TODO: to flip or not to flip?
        //                 flip_y: false,
        //                 ..Default::default()
        //             },
        //         );
        //         // mq::draw_rectangle(previous_x, floor_level - (h / 2.0), w, h, mq::WHITE);

        //         let fog_brightness = (2.0 * ray_hit.world_distance / VIEW_DISTANCE - 1.0).max(0.0);
        //         let fog_color = mq::Color::new(
        //             BACKGROUND_COLOR.r,
        //             BACKGROUND_COLOR.g,
        //             BACKGROUND_COLOR.b,
        //             fog_brightness,
        //         );

        //         mq::draw_rectangle(previous_x, floor_level - (h / 2.0), w, h, fog_color);
        //     }

        //     previous_x = x;
        // }

        // crosshair
        mq::draw_line(
            WINDOW_WIDTH as f32 * (3.0 / 4.0) - 10.0,
            WINDOW_HEIGHT as f32 / 2.0,
            WINDOW_WIDTH as f32 * (3.0 / 4.0) + 10.0,
            WINDOW_HEIGHT as f32 / 2.0,
            2.0,
            mq::BLACK,
        );
        mq::draw_line(
            WINDOW_WIDTH as f32 * (3.0 / 4.0),
            WINDOW_HEIGHT as f32 / 2.0 - 10.0,
            WINDOW_WIDTH as f32 * (3.0 / 4.0),
            WINDOW_HEIGHT as f32 / 2.0 + 10.0,
            2.0,
            mq::BLACK,
        );

        // text background
        mq::draw_rectangle(0.0, 0.0, 140.0, 35.0, mq::Color::new(1.0, 1.0, 1.0, 1.0));

        // text
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
