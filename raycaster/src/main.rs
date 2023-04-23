use macroquad::prelude as mq;

const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 512;

const MAP_WIDTH: u32 = 8;
const MAP_HEIGHT: u32 = 8;
const TILE_SIZE: u32 = 64;

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
            self.direction = mq::Vec2::new(self.angle.cos(), self.angle.sin());
        }
        if mq::is_key_down(mq::KeyCode::Right) {
            self.angle += 3.0 * delta;
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
        }
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
        window_resizable: true,
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

        // mq::draw_line(40.0, 40.0, 100.0, 200.0, 15.0, mq::BLUE);
        // mq::draw_rectangle(mq::screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, mq::GREEN);
        // mq::draw_circle(mq::screen_width() - 30.0, mq::screen_height() - 30.0, 15.0, mq::YELLOW);
        // mq::draw_text("HELLO", 20.0, 20.0, 20.0, mq::DARKGRAY);

        mq::draw_text(
            format!("FPS: {}", mq::get_fps()).as_str(),
            5.,
            15.,
            20.,
            mq::BLUE,
        );
        mq::draw_text(
            format!("DELTA: {:.2}", delta / 1000.0).as_str(),
            5.,
            30.,
            20.,
            mq::BLUE,
        );

        mq::next_frame().await
    }
}
