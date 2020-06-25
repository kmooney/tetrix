use rand::Rng;
const VERSION: f32 = 0.01;
const WIDTH: usize  = 10;
const HEIGHT: usize = 25;

#[derive(Debug)]
pub enum Orientation {
    Up, Down, Left, Right,
}

#[derive(Debug)]
pub enum Shape {
    Eye, El, ElInv, Square, Zee, ZeeInv,
}

type ShapeMat = [[bool; 4]; 4];

impl Drop for Shape {
    fn drop(&mut self) {
        println!("Shape {:?} is destructing.", self);
    }
}

#[derive(Debug)]
pub struct Point {
    x: usize, 
    y: usize,
}

impl Shape {
    fn random() -> Shape {
        match rand::thread_rng().gen_range(0, 6) {
            0 => Shape::Eye,
            1 => Shape::El,
            2 => Shape::ElInv,
            3 => Shape::Square,
            4 => Shape::Zee,
            5 => Shape::ZeeInv,
            _ => panic!("wtf value is out of range")
        }	
    }

    fn to_a(&self, o: &Orientation) -> ShapeMat {
        match self {
            Shape::Eye => match o {
                Orientation::Left | Orientation::Right => [
                    [false, false, false, false],
                    [false, false, false, false],
                    [false, false, false, false],
                    [true,  true,  true,  true],
                ],
                Orientation::Up | Orientation::Down => [
                    [true, false, false, false],
                    [true, false, false, false],
                    [true, false, false, false],
                    [true, false, false, false],
                ]
            },
            Shape::El => match o {
                Orientation::Up => [
                    [false, false, false, false],
                    [true,  false, false, false],
                    [true,  false, false, false],
                    [true,  true,  false, false],
                ],
                Orientation::Left => [
                    [false, false, false, false],
                    [false, false, false, false],
                    [false, false, true, false],
                    [true,  true,  true, false],
                ],
                Orientation::Down => [
                    [true, true, false, false],
                    [false, true, false, false],
                    [false, true, false, false],
                    [false,  true,  false, false],
                ],
                Orientation::Right => [
                    [false, false, false, false],
                    [false, false, false, false],
                    [true, true, true, false],
                    [true,  false,  false, false],
                ],
            },
            Shape::ElInv => match o {
                Orientation::Up => [
                    [false, false, false, false],
                    [false,  true, false, false],
                    [false,  true, false, false],
                    [true,  true,  false, false],
                ],
                Orientation::Left => [
                    [false, false, false, false],
                    [false, false, false, false],
                    [true, true, true, false],
                    [false,  false,  true, false],
                ],
                Orientation::Down => [
                    [false, false, false, false],
                    [true, true, false, false],
                    [true, false, false, false],
                    [true,  false,  false, false],
                ],
                Orientation::Right => [
                    [false, false, false, false],
                    [false, false, false, false],
                    [true, false, false, false],
                    [true,  true,  true, false],
                ],
            },
            Shape::Square => match o {
                Orientation::Up | Orientation::Down | Orientation::Left | Orientation::Right => [
                    [false, false, false, false],
                    [false,  false, false, false],
                    [true,  true, false, false],
                    [true,  true,  false, false],
                ]
            },
            Shape::Zee => match o {
                Orientation::Up | Orientation::Down => [
                    [false, false, false, false],
                    [true,  false, false, false],
                    [true,  true, false, false],
                    [false,  true,  false, false],
                ],
                Orientation::Left | Orientation::Right => [
                    [false, false, false, false],
                    [false, false, false, false],
                    [false, true, true, false],
                    [true,  true,  false, false],
                ]
            }
            Shape::ZeeInv => match o {
                Orientation::Up | Orientation::Down => [
                    [false, false, false, false],
                    [false,  true, false, false],
                    [true,  true, false, false],
                    [true,  false,  false, false],
                ],
                Orientation::Left | Orientation::Right => [
                    [false, false, false, false],
                    [false, false, false, false],
                    [true, true, false, false],
                    [false,  true,  true, false],
                ]
            }
        }
    }

} 

pub struct ShapeController {
    orientation: Orientation,
    position: Point,
    shape: Shape,
}

impl ShapeController {
    pub fn new() -> ShapeController {
        let s = Shape::random();
        let position = match s {
            Shape::Eye => Point{x: 3, y: 21},
            _ => Point{x: 4, y:21},
        };

        ShapeController {
            orientation: Orientation::Up,
            position: position,
            shape: s
        }
    }

    pub fn shape(&self) -> &Shape {
        return &self.shape;
    }

    pub fn position(&self) -> &Point {
        return &self.position;
    }

    pub fn orientation(&self) -> &Orientation {
        return &self.orientation;
    }

    pub fn down(&mut self) {
        if self.position.y > 0 {
            self.position.y -= 1;
        }
    }

    pub fn rotate_cw(&mut self) {
        self.orientation = match self.orientation {
            Orientation::Up => Orientation::Right,
            Orientation::Right => Orientation::Down,
            Orientation::Down => Orientation::Left,
            Orientation::Left => Orientation::Up
        }
    }

    pub fn rotate_ccw(&mut self) {
        self.orientation = match self.orientation {
            Orientation::Up => Orientation::Left,
            Orientation::Left => Orientation::Down,
            Orientation::Down => Orientation::Right,
            Orientation::Right => Orientation::Up
        }
    }

}

struct Board([[bool; WIDTH]; HEIGHT]);

impl Board {
    pub fn report(&self) -> String {
        let mut board_report = String::new();
        board_report.push_str(&format!("[  ]----{:02}----\n", HEIGHT));
        for y in (0..HEIGHT).rev() {
            let row = self.0[y];
            board_report.push_str(&format!("{:02} ", y));
            for cell in row.iter() {
                board_report.push_str(match cell {
                    true => "x",
                    false => "_",
                })
            }
            board_report.push_str("\n");
        }
        board_report.push_str("-------------\n");
        board_report.push_str("  |0123456789\n");
        board_report
    }

    pub fn occupy(&mut self, c: &ShapeController) {
        let m = c.shape.to_a(c.orientation());
        let p = c.position();
        for y in 0..4 {
            for x in 0..4 {
                // 0 is the bottom of the board
                // so invert the y coordinate for this
                // shape
                self.0[y+p.y][x+p.x] |= m[3-y][x];
            }
        }
    }

    pub fn vacate(&mut self, c: &ShapeController) {
        let m = &c.shape.to_a(&c.orientation);
        let p = c.position();
        for y in 0..4 {
            for x in 0..4 {
                if self.0[y+p.y][x+p.x] && m[3-y][x] {
                    self.0[y+p.y][x+p.x] = false;
                }
            }
        }
    }
}

#[derive(Debug)]
enum GameState {New, Playing, Over}

pub struct Game {
    board: Board,
    score: u32,
    shape_controller: ShapeController,
    next_shape: Shape,
    hold_shape: Option<Shape>,
    state: GameState,
}

impl Game {
    pub fn new() -> Game {
        Game {
            board: Board([[false; WIDTH]; HEIGHT]),
            score: 0,
            shape_controller: ShapeController::new(),
            next_shape: Shape::random(),
            hold_shape: None,
            state: GameState::New,
        } 
    }

    pub fn shape_controller(&mut self) -> &mut ShapeController {
        return &mut self.shape_controller;
    }

    pub fn reset_board(&mut self) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                self.board.0[y][x] = false;
            }
        }   
    }

    pub fn setup_board(&mut self, config: Vec<Vec<bool>>, position: Point, overwrite: bool) {
        let mut x;
        let mut y = 0;

        for row in config.iter() {
            x = 0;
            for cell in row.iter() {
                println!("x {}, y {}", x, y);
                if overwrite {
                    self.board.0[y + position.y][x + position.x] = *cell;
                } else {
                    self.board.0[y + position.y][x + position.x] |= *cell;
                }
                x += 1;
            }
            y += 1;
        }
    }

    pub fn report(&self) -> String {
        let current_piece_status = format!("{:?}", self.shape_controller.position);
        let current_piece_orientation = format!("shape = {:?}, orientation = {:?}", self.shape_controller.shape(), self.shape_controller.orientation);
        return String::from(format!("T E T R I X version {}\n{}\n{}\n{}\nscore: {}\nstate:{:?}\n", VERSION, current_piece_status, current_piece_orientation, self.board.report(), self.score, self.state))
    }

    fn check_collision(&self, s: &Shape, p: &Point) -> bool {
        let m = s.to_a(&self.shape_controller.orientation);
        for y in 0..4 {
            for x in 0..4 {
                let cell = m[3-y][x];
                if cell && self.board.0[y + p.y - 1][x + p.x] {
                    println!("board collision! make a new shape.");
                    return true;
                }
            }
        }
        return false;
    }

    pub fn check_shape(&self) -> bool {
        let s = &self.shape_controller.shape;
        let p = &self.shape_controller.position;
        if p.y == 0 {
            println!("p.y is zero!  make a new shape.");
            return true;
        }
        return self.check_collision(s, p);
    }

    pub fn check_over(&self) -> bool {
        let s = &self.shape_controller.shape;
        let p = &self.shape_controller.position;
        return p.y >= 20 && self.check_collision(s, p);
    }

    pub fn next(&mut self) {
        match self.state { 
            GameState::Playing => {},
            _ => return,
        }
        self.board.vacate(&self.shape_controller);

        if self.check_shape() {
            if self.check_over() {
                self.state = GameState::Over;
            }
            self.board.occupy(&self.shape_controller);
            self.shape_controller = ShapeController::new();
        }
    
        self.shape_controller.down();
        self.board.occupy(&self.shape_controller);
        println!("made next state");
    }

    pub fn start(&mut self) {
        self.state = GameState::Playing;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game() {
        // when the game starts, there should be a shape controller with the current shape
        // and there should be a next shape.  
        // there should be no "hold" shape

        let mut g = Game::new();
        match g.hold_shape {
            None => assert!(true),
            _ => assert!(false, "Hold shape should be unset at start")
        }
        let config = vec![
            vec![false, true, false, true, false, false, true],
            vec![false, true, false, true, false, false, true],
            vec![false, true, false, true, false, false, true],
            vec![false, true,  true, true, false, false, true],
            vec![false, true, false, true, false, false, true],
            vec![false, true, false, true, false, false, true],
            vec![false, true, false, true, false, false, true],
        ];
        g.setup_board(config, Point{x: 1, y: 3}, true);
        println!("{}", g.report());
        g.reset_board();
        println!("{}", g.report());
        let mut trues = 0;
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                match g.board.0[y][x] {
                    true => trues += 1,
                    false => {}
                }
            }
        }
        assert_eq!(trues, 0, "no boxes after board reset!");
    }
}