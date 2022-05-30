use ggez::event::KeyCode;
use ggez::{event, graphics, Context, GameResult};

use std::collections::LinkedList;
use std::time::{Duration, Instant};

use rand::Rng;

const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

const GRID_SIZE: (i16, i16) = (25, 25);
const GRID_CELL_SIZE: (i16, i16) = (25, 25);

const SCREEN_SIZE: (u32, u32) = (
    GRID_SIZE.0 as u32 * GRID_CELL_SIZE.0 as u32,
    GRID_SIZE.1 as u32 * GRID_CELL_SIZE.1 as u32,
);

const FRAMES_PER_SECOND: f32 = 8.0;
const MS_PER_FRAME: u64 = (1.0 / FRAMES_PER_SECOND * 1000.0) as u64;

trait ModulusSigned {
    fn modulus_signed(&self, n: Self) -> Self;
}

impl<T> ModulusSigned for T
where
    T: std::ops::Add<Output = T> + std::ops::Rem<Output = T> + Clone,
{
    fn modulus_signed(&self, n: T) -> T {
        (self.clone() % n.clone() + n.clone()) % n
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn inverse(&self) -> Self {
        match *self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

struct KeyboardListener {}

impl KeyboardListener {
    fn from_keycode(key: KeyCode) -> Option<Direction> {
        match key {
            KeyCode::Up | KeyCode::W => Some(Direction::Up),
            KeyCode::Down | KeyCode::S => Some(Direction::Down),
            KeyCode::Left | KeyCode::A => Some(Direction::Left),
            KeyCode::Right | KeyCode::D => Some(Direction::Right),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct GridPosition {
    x: i16,
    y: i16,
}

impl GridPosition {
    fn new(x: i16, y: i16) -> Self {
        GridPosition { x, y }
    }

    fn random(max_x: i16, max_y: i16) -> Self {
        let mut rng = rand::thread_rng();

        (rng.gen_range(0..max_x), rng.gen_range(0..max_y)).into()
    }

    fn new_from_move(position: GridPosition, direction: Direction) -> Self {
        match direction {
            Direction::Up => {
                GridPosition::new(position.x, (position.y - 1).modulus_signed(GRID_SIZE.1))
            }
            Direction::Down => {
                GridPosition::new(position.x, (position.y + 1).modulus_signed(GRID_SIZE.1))
            }
            Direction::Left => {
                GridPosition::new((position.x - 1).modulus_signed(GRID_SIZE.0), position.y)
            }
            Direction::Right => {
                GridPosition::new((position.x + 1).modulus_signed(GRID_SIZE.0), position.y)
            }
        }
    }
}

impl From<GridPosition> for graphics::Rect {
    fn from(position: GridPosition) -> Self {
        graphics::Rect::new_i32(
            position.x as i32 * GRID_CELL_SIZE.0 as i32,
            position.y as i32 * GRID_CELL_SIZE.1 as i32,
            GRID_CELL_SIZE.0 as i32,
            GRID_CELL_SIZE.1 as i32,
        )
    }
}

impl From<(i16, i16)> for GridPosition {
    fn from(position: (i16, i16)) -> Self {
        GridPosition {
            x: position.0,
            y: position.1,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Segment {
    position: GridPosition,
}

impl Segment {
    fn new(position: GridPosition) -> Self {
        Segment { position }
    }
}

struct Food {
    position: GridPosition,
}

impl Food {
    fn new(position: GridPosition) -> Self {
        Food { position }
    }

    fn draw(&self, context: &mut Context) -> GameResult {
        let mesh = graphics::MeshBuilder::new()
            .rectangle(
                graphics::DrawMode::fill(),
                self.position.into(),
                graphics::Color::new(0.0, 0.0, 1.0, 1.0),
            )?
            .build(context)?;

        graphics::draw(context, &mesh, graphics::DrawParam::default())?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
enum Collision {
    Food,
    Itself,
}

struct Player {
    head: Segment,
    body: LinkedList<Segment>,
    direction: Direction,
    collision: Option<Collision>,
    last_update_direction: Direction,
}

impl Player {
    fn new(position: GridPosition) -> Self {
        let mut body = LinkedList::new();
        body.push_back(Segment::new((position.x - 1, position.y).into()));

        Player {
            head: Segment::new(position),
            body,
            direction: Direction::Right,
            collision: None,
            last_update_direction: Direction::Right,
        }
    }

    fn eats(&self, food: &Food) -> bool {
        self.head.position == food.position
    }

    fn collides_with_itself(&self) -> bool {
        for segment in self.body.iter() {
            if self.head.position == segment.position {
                return true;
            }
        }
        false
    }

    fn update(&mut self, food: &Food) {
        let new_head_position = GridPosition::new_from_move(self.head.position, self.direction);
        let new_head = Segment::new(new_head_position);

        self.body.push_front(self.head);
        self.head = new_head;

        if self.collides_with_itself() {
            self.collision = Some(Collision::Itself);
        } else if self.eats(food) {
            self.collision = Some(Collision::Food);
        } else {
            self.collision = None;
        }

        if self.collision.is_none() {
            self.body.pop_back();
        }

        self.last_update_direction = self.direction;
    }

    fn draw(&self, context: &mut Context) -> GameResult {
        for segment in self.body.iter() {
            let mesh = graphics::MeshBuilder::new()
                .rectangle(
                    graphics::DrawMode::fill(),
                    segment.position.into(),
                    graphics::Color::new(1.0, 0.5, 0.0, 1.0),
                )?
                .build(context)?;
            graphics::draw(context, &mesh, graphics::DrawParam::default())?;
        }
        let mesh = graphics::MeshBuilder::new()
            .rectangle(
                graphics::DrawMode::fill(),
                self.head.position.into(),
                graphics::Color::new(1.0, 0.0, 0.0, 1.0),
            )?
            .build(context)?;

        graphics::draw(context, &mesh, graphics::DrawParam::default())?;
        Ok(())
    }
}

struct GameState {
    player: Player,
    food: Food,
    game_over: bool,
    last_update: Instant,
}

impl GameState {
    fn new() -> GameResult<Self> {
        let player_position = (GRID_SIZE.0 / 4, GRID_SIZE.1 / 2).into();
        let food_position = GridPosition::random(GRID_SIZE.0, GRID_SIZE.1);

        Ok(GameState {
            player: Player::new(player_position),
            food: Food::new(food_position),
            game_over: false,
            last_update: Instant::now(),
        })
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if Instant::now() - self.last_update < Duration::from_millis(MS_PER_FRAME) {
            return Ok(());
        }
        if self.game_over {
            self.player = Player::new((GRID_SIZE.0 / 4, GRID_SIZE.1 / 2).into());
            self.food = Food::new(GridPosition::random(GRID_SIZE.0, GRID_SIZE.1));
            self.game_over = false;

            return Ok(());
        }
        self.player.update(&self.food);

        if let Some(collision) = self.player.collision {
            match collision {
                Collision::Food => {
                    let new_food_position = GridPosition::random(GRID_SIZE.0, GRID_SIZE.1);
                    self.food.position = new_food_position;
                }

                Collision::Itself => {
                    self.game_over = true;
                }
            }
        }
        self.last_update = Instant::now();
        Ok(())
    }

    fn draw(&mut self, context: &mut Context) -> GameResult {
        graphics::clear(context, GREEN.into());

        self.player.draw(context)?;
        self.food.draw(context)?;

        graphics::present(context)?;

        ggez::timer::yield_now();

        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: event::KeyMods,
        _repeat: bool,
    ) {
        if let Some(direction) = KeyboardListener::from_keycode(keycode) {
            if direction.inverse() != self.player.last_update_direction {
                self.player.direction = direction;
            }
        }
    }
}

pub fn run() -> GameResult {
    let (context, event_loop) = ggez::ContextBuilder::new("Snake Game", "DevAles")
        .window_setup(ggez::conf::WindowSetup::default().title("Snake Game"))
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(SCREEN_SIZE.0 as f32, SCREEN_SIZE.1 as f32),
        )
        .build()
        .expect("Failed to build context!");

    let state = GameState::new()?;
    event::run(context, event_loop, state)
}
