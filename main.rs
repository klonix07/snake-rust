use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::{self, Color, DrawMode, Mesh, Rect, Text};
use ggez::{Context, ContextBuilder, GameResult};
use rand::Rng;

/// Represents a point on the game grid.
#[derive(Clone, Copy, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

/// Represents the possible directions for snake movement.
#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// The main game state struct containing all necessary fields.
struct SnakeGame {
    // The snake is represented as a vector of Points; the first element is the head.
    snake: Vec<Point>,
    // Current movement direction.
    direction: Direction,
    // Holds the next valid direction (set via user input) to avoid mid-frame reversal.
    next_direction: Direction,
    // The current food position.
    food: Point,
    // The player’s score.
    score: u32,
    // Grid dimensions (number of cells horizontally and vertically).
    grid_width: i32,
    grid_height: i32,
    // Timer used to control snake movement timing.
    move_timer: f32,
    // Time between snake moves (in seconds).
    move_period: f32,
    // Game-over flag.
    game_over: bool,
}

impl SnakeGame {
    /// Creates a new game state with an initial snake position and randomly placed food.
    fn new(grid_width: i32, grid_height: i32) -> SnakeGame {
        // Start the snake in the center of the grid.
        let init_pos = Point {
            x: grid_width / 2,
            y: grid_height / 2,
        };
        let snake = vec![init_pos];
        let food = SnakeGame::generate_food(&snake, grid_width, grid_height);
        SnakeGame {
            snake,
            direction: Direction::Right,
            next_direction: Direction::Right,
            food,
            score: 0,
            grid_width,
            grid_height,
            move_timer: 0.0,
            move_period: 0.2, // Move every 0.2 seconds.
            game_over: false,
        }
    }

    /// Generates a new food location that is not currently occupied by the snake.
    fn generate_food(snake: &Vec<Point>, grid_width: i32, grid_height: i32) -> Point {
        let mut rng = rand::thread_rng();
        loop {
            let food = Point {
                x: rng.gen_range(0..grid_width),
                y: rng.gen_range(0..grid_height),
            };
            // Ensure the food does not appear on the snake.
            if !snake.contains(&food) {
                return food;
            }
        }
    }

    /// Updates the snake’s position and checks for collisions and food consumption.
    fn update_snake(&mut self) {
        if self.game_over {
            return;
        }

        // Update the current direction from the next_direction (set by user input).
        self.direction = self.next_direction;

        // Compute the new head position based on the current direction.
        let mut new_head = *self
            .snake
            .first()
            .expect("Snake should always have at least one segment");
        match self.direction {
            Direction::Up => new_head.y -= 1,
            Direction::Down => new_head.y += 1,
            Direction::Left => new_head.x -= 1,
            Direction::Right => new_head.x += 1,
        }

        // Check for collision with the boundaries of the grid.
        if new_head.x < 0
            || new_head.x >= self.grid_width
            || new_head.y < 0
            || new_head.y >= self.grid_height
        {
            self.game_over = true;
            return;
        }

        // Check for collision with the snake's own body.
        if self.snake.contains(&new_head) {
            self.game_over = true;
            return;
        }

        // Insert the new head position at the beginning of the snake vector.
        self.snake.insert(0, new_head);

        // Check if the snake has eaten the food.
        if new_head == self.food {
            self.score += 1;
            // Spawn new food at a random location.
            self.food = SnakeGame::generate_food(&self.snake, self.grid_width, self.grid_height);
        } else {
            // Remove the tail segment to move the snake forward.
            self.snake.pop();
        }
    }
}

/// Implementing ggez’s EventHandler trait to define game behavior.
impl EventHandler for SnakeGame {
    /// Updates the game logic on each frame.
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Get the time elapsed since the last update.
        let dt = ggez::timer::delta(ctx).as_secs_f32();
        self.move_timer += dt;
        // Move the snake when the move_timer exceeds move_period.
        if self.move_timer > self.move_period {
            self.move_timer = 0.0;
            self.update_snake();
        }
        Ok(())
    }

    /// Draws the current game state.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Clear the screen to black.
        graphics::clear(ctx, Color::from_rgb(0, 0, 0));

        let cell_size = 20.0;
        // Draw each segment of the snake.
        for segment in &self.snake {
            let rectangle = Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                Rect::new_i32(segment.x * cell_size as i32, segment.y * cell_size as i32, cell_size as i32, cell_size as i32),
                Color::from_rgb(0, 255, 0),
            )?;
            graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
        }

        // Draw the food as a red square.
        let food_rect = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new_i32(self.food.x * cell_size as i32, self.food.y * cell_size as i32, cell_size as i32, cell_size as i32),
            Color::from_rgb(255, 0, 0),
        )?;
        graphics::draw(ctx, &food_rect, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;

        // Draw the current score in the top-left corner.
        let score_text = Text::new(format!("Score: {}", self.score));
        graphics::draw(ctx, &score_text, (ggez::mint::Point2 { x: 10.0, y: 10.0 }, Color::from_rgb(255, 255, 255)))?;

        // If the game is over, display a game-over message.
        if self.game_over {
            let over_text = Text::new("Game Over! Press R to Restart");
            let (w, h) = graphics::drawable_size(ctx);
            let dest_point = ggez::mint::Point2 { x: w / 2.0 - 100.0, y: h / 2.0 };
            graphics::draw(ctx, &over_text, (dest_point, Color::from_rgb(255, 255, 255)))?;
        }

        // Present the drawn frame on the screen.
        graphics::present(ctx)?;
        Ok(())
    }

    /// Handles keyboard input for controlling the snake and restarting the game.
    fn key_down_event(&mut self, _ctx: &mut Context, key: KeyCode, _mods: KeyMods, _repeat: bool) {
        // Map key presses to direction changes.
        let new_direction = match key {
            KeyCode::Up => Some(Direction::Up),
            KeyCode::Down => Some(Direction::Down),
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
            // If the game is over, pressing 'R' restarts the game.
            KeyCode::R if self.game_over => {
                *self = SnakeGame::new(self.grid_width, self.grid_height);
                None
            },
            _ => None,
        };

        if let Some(nd) = new_direction {
            // Prevent the snake from reversing directly onto itself.
            if (self.direction == Direction::Up && nd != Direction::Down)
                && (self.direction == Direction::Down && nd != Direction::Up)
                || (self.direction == Direction::Left && nd != Direction::Right)
                || (self.direction == Direction::Right && nd != Direction::Left)
            {
                // A more robust check: ensure that the new direction isn’t directly opposite.
                if !(self.direction == Direction::Up && nd == Direction::Down)
                    && !(self.direction == Direction::Down && nd == Direction::Up)
                    && !(self.direction == Direction::Left && nd == Direction::Right)
                    && !(self.direction == Direction::Right && nd == Direction::Left)
                {
                    self.next_direction = nd;
                }
            } else if 
                (self.direction == Direction::Up && nd != Direction::Down)
                || (self.direction == Direction::Down && nd != Direction::Up)
                || (self.direction == Direction::Left && nd != Direction::Right)
                || (self.direction == Direction::Right && nd != Direction::Left)
            {
                self.next_direction = nd;
            }
        }
    }
}

/// The main function sets up the game window and starts the event loop.
fn main() -> GameResult {
    // Create a new ggez Context and event loop.
    let (mut ctx, event_loop) = ContextBuilder::new("snake_game", "Author")
        .window_setup(ggez::conf::WindowSetup::default().title("Snake Game"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(400.0, 400.0))
        .build()?;

    // Our grid is 20x20 cells.
    let game = SnakeGame::new(20, 20);
    // Run the game event loop.
    event::run(ctx, event_loop, game)
}
