#[cfg(feature = "buddy-alloc")]
mod alloc;
mod wasm4;

use wasm4::*;

#[derive(Clone, Copy)]
pub struct Point {
    x: i8,
    y: i8,
}

pub struct Snake {
    body: [Point; get_max_points() as usize],
    direction: Point,
}

pub struct Fruit {
    location: Point,
    sprite: [u8; 16],
}

pub struct Game {
    snake: Snake,
    fruit: Fruit,
    fruit_count: u16,
    frame_count: u32,
    prev_input: u8,
    // The snake does not update at each cycle so we need to prevent that multiple inputs lead to unwanted behavior
    // e.g snake goes from left to right immediately and eats her "neck"
    processing_input: bool,
}

impl Point {
    pub const fn new(x: i8, y: i8) -> Self {
        Point { x, y }
    }

    pub fn equals(&self, other: Point) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Snake {
    pub const fn new() -> Self {
        let mut body: [Point; get_max_points() as usize] =
            [Point::new(-1, -1); get_max_points() as usize];
        body[0] = Point::new(0, 0);
        body[1] = Point::new(1, 0);
        body[2] = Point::new(2, 0);
        Snake {
            body,
            direction: Point::new(1, 0),
        }
    }

    pub fn update(&mut self, grow: bool) {
        let parts = self.body.clone();
        for i in 0..parts.len() {
            if i == 0 {
                let mut x = self.body[i].x - self.direction.x;
                let mut y = self.body[i].y - self.direction.y;
                let max = (RESOLUTION / SPRITE_SIZE) as i8 - 1;
                // If snake moves out of the "game area" set its position to the opposite side
                if x < 0 {
                    x = max;
                } else if x > max {
                    x = 0;
                }
                if y < 0 {
                    y = max;
                } else if y > max {
                    y = 0;
                }
                self.body[i] = Point::new(x, y);
            } else if self.body[i].x == -1 {
                if grow == true {
                    self.body[i] = parts[i - 1];
                }
                break;
            } else {
                self.body[i] = parts[i - 1];
            }
        }
    }

    pub fn draw(&self) {
        let mut i = 0;
        while i < self.body.len() && self.body[i].x > -1 {
            set_draw_colors(0x43);
            if i == 0 {
                set_draw_colors(0x4);
            }
            rect(self.body[i].x as i32 * 8, self.body[i].y as i32 * 8, 8, 8);
            i += 1;
        }
    }

    pub fn left(&mut self) {
        if self.direction.x == 0 {
            self.direction.x = 1;
            self.direction.y = 0;
        }
    }

    pub fn right(&mut self) {
        if self.direction.x == 0 {
            self.direction.x = -1;
            self.direction.y = 0;
        }
    }

    pub fn up(&mut self) {
        if self.direction.y == 0 {
            self.direction.x = 0;
            self.direction.y = 1;
        }
    }

    pub fn down(&mut self) {
        if self.direction.y == 0 {
            self.direction.x = 0;
            self.direction.y = -1;
        }
    }
}

impl Fruit {
    pub const fn new() -> Self {
        Fruit {
            location: Point::new(-1, -1),
            sprite: [
                0x00, 0xa0, 0x02, 0x00, 0x0e, 0xf0, 0x36, 0x5c, 0xd6, 0x57, 0xd5, 0x57, 0x35, 0x5c,
                0x0f, 0xf0,
            ],
        }
    }

    pub fn draw(&self) {
        set_draw_colors(0x4320);
        blit(
            &self.sprite,
            self.location.x as i32 * 8,
            self.location.y as i32 * 8,
            8,
            8,
            BLIT_2BPP,
        );
    }
}

impl Game {
    pub const fn new() -> Self {
        Game {
            snake: Snake::new(),
            fruit: Fruit::new(),
            fruit_count: 0,
            frame_count: 0,
            prev_input: 0,
            processing_input: false,
        }
    }

    fn update(&mut self) {
        self.frame_count += 1;
    }

    fn input(&mut self) {
        if self.processing_input == true {
            // Do nothing if there already was a valid input that's not yet processed
            return;
        }
        let gamepad: u8 = get_gamepad();
        let just_pressed: u8 = gamepad & (gamepad ^ self.prev_input);

        if just_pressed & BUTTON_LEFT != 0 {
            self.snake.left();
        } else if just_pressed & BUTTON_RIGHT != 0 {
            self.snake.right();
        } else if just_pressed & BUTTON_UP != 0 {
            self.snake.up();
        } else if just_pressed & BUTTON_DOWN != 0 {
            self.snake.down();
        }

        if just_pressed != 0 {
            self.processing_input = true;
        }
        self.prev_input = gamepad;
    }

    pub fn check_fruit_collision(&mut self) -> bool {
        let mut collision = false;
        if self.snake.body[0].equals(self.fruit.location) {
            collision = true;
        }
        collision
    }

    pub fn check_snake_collision(&mut self) -> bool {
        for i in 1..self.snake.body.len() {
            // Skip 0 because it is the head
            if self.snake.body[0].equals(self.snake.body[i]) {
                return true;
            }
        }
        false
    }

    pub fn place_random_fruit(&mut self) {
        let max_col_row = (RESOLUTION / SPRITE_SIZE) as i8;
        let mut available_locations: Vec<Point> = Vec::new();
        // We do not want wo place the fruit on the snake so we gather all available locations
        // that do not have a body part of the
        for y in 0..max_col_row {
            for x in 0..max_col_row {
                if self
                    .snake
                    .body
                    .iter()
                    .find(|v| v.x == x && v.y == y)
                    .is_some()
                {
                    continue;
                }
                available_locations.push(Point::new(x, y));
            }
        }
        self.fruit.location = get_random_location(self.frame_count, available_locations);
    }
}

const RESOLUTION: u8 = 160;
const SPRITE_SIZE: u8 = 8;
static mut GAME: Game = Game::new();

#[no_mangle]
fn start() {
    set_palette();
    unsafe {
        GAME.place_random_fruit();
    }
}

#[no_mangle]
fn update() {
    let mut game = unsafe { &mut GAME };
    let game_over = game.check_snake_collision();
    if game_over {
        set_draw_colors(0x4);
        text("Game Over!", 44, 60);
        text(format!("Score: {}", game.fruit_count), 44, 74);
    } else {
        game.update();
        game.input();
        game.fruit.draw();
        if game.frame_count % 15 == 0 {
            let food_collision = game.check_fruit_collision();
            game.snake.update(food_collision);
            if food_collision {
                game.place_random_fruit();
                game.fruit_count += 1;
            }
            game.processing_input = false;
        }
        game.snake.draw();
    }
}

fn get_random_location(frame_count: u32, available_locations: Vec<Point>) -> Point {
    // It is just a simple random number generation depending on the frame_count
    // for this game it is good enough
    let rng = (frame_count * frame_count + 32415412) % available_locations.len() as u32;
    return *available_locations.get(rng as usize).unwrap();
}

fn get_gamepad() -> u8 {
    unsafe { *GAMEPAD1 }
}

fn set_palette() {
    unsafe { *PALETTE = [0xfbf7f3, 0xe5b083, 0x426e5d, 0x20283d] };
}

fn set_draw_colors(color: u16) {
    unsafe { *DRAW_COLORS = color };
}

const fn get_max_points() -> u32 {
    ((RESOLUTION / SPRITE_SIZE) as u32).pow(2)
}
