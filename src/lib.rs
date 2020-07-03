mod shape;
mod shape_controller;
mod board;
use board::Board;
use shape_controller::{ShapeController, Direction};
use shape::{Shape, Point, Orientation};

const VERSION: f32 = 0.01;
const WIDTH: usize  = 10;
const HEIGHT: usize = 25;

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

    pub fn get_shape_controller(&self) -> &ShapeController {
        return &self.shape_controller
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
        let config_height = config.len();
        for row in config.iter() {
            x = 0;
            for cell in row.iter() {
                if overwrite {
                    self.board.0[config_height - y + position.y - 1][x + position.x] = *cell;
                } else {
                    self.board.0[config_height - y + position.y - 1][x + position.x] |= *cell;
                }
                x += 1;
            }
            y += 1;
        }
    }

    pub fn report(&self) -> String {
        let current_piece_status = format!("{:?}", self.get_shape_controller().position());
        let current_piece_orientation = format!("shape = {:?}, orientation = {:?}", self.shape_controller.shape(), self.shape_controller.orientation());
        return String::from(format!("T E T R I X version {}\n{}\n{}\n{}\nscore: {}\nstate:{:?}\n", VERSION, current_piece_status, current_piece_orientation, self.board.report(), self.score, self.state))
    }

    fn check_collision(&self, s: &Shape, p: &Point) -> bool {
        let m = s.to_mat(&self.get_shape_controller().orientation());
        for y in 0..4 {
            for x in 0..4 {
                let cell = m[3-y][x];
                if cell && self.board.0[y + p.y - 1][x + p.x] {
                    return true;
                }
            }
        }
        return false;
    }

    pub fn check_shape(&self) -> bool {
        let s = &self.get_shape_controller().shape();
        let p = &self.get_shape_controller().position();
        if p.y == 0 {
            return true;
        }
        return self.check_collision(s, p);
    }

    pub fn check_over(&self) -> bool {
        let s = &self.get_shape_controller().shape();
        let p = &self.get_shape_controller().position();
        return p.y >= 20 && self.check_collision(s, p);
    }

    pub fn rotate(&mut self, direction: Direction) {
        let b = &mut self.board;
        let c = &mut self.shape_controller;
        b.vacate(c);
        c.rotate(direction, b);
        b.occupy(c);
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
        g.reset_board();
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

    #[test]
    fn rotate() {
        let mut g = Game::new();
        g.shape_controller().set_shape(Shape::El);
        g.shape_controller().set_position(Point::new(3,3));
        g.start();
        g.rotate(Direction::Ccw);
        assert!(g.board.0[3][3]);
        assert!(g.board.0[3][4]);
        assert!(g.board.0[3][5]);
        assert!(g.board.0[4][5]);
        assert!(g.board.0[3][6] == false);
    }

    #[test]
    fn wall_kick_l() {
        let mut g = Game::new();
        g.shape_controller().set_shape(Shape::El);
        g.shape_controller().set_position(Point::new(0, 3));
        g.start();
        let b = &mut g.board;
        b.occupy(&g.shape_controller);
        assert!(g.board.0[3][0], "box 1 in the wrong spot!");
        assert!(g.board.0[3][1]);
        assert!(g.board.0[4][0]);
        assert!(g.board.0[5][0]);
        g.rotate(Direction::Ccw);
        assert!(g.board.0[3][0]);
        assert!(g.board.0[3][1]);
        assert!(g.board.0[3][2]);
        assert!(g.board.0[4][2]);
    }

    #[test]
    fn flush_wall_r() {
        let mut g = Game::new();
        g.shape_controller.set_shape(Shape::El);
        g.shape_controller.set_position(Point::new(8, 3));
        g.shape_controller.set_orientation(Orientation::Up);
        g.start();
        let b = &mut g.board;
        b.occupy(&g.shape_controller);
        assert!(g.board.0[3][8], "box 1 in the wrong spot!");
        assert!(g.board.0[4][8]);
        assert!(g.board.0[5][8]);
        assert!(g.board.0[3][9]);
    }

    #[test]
    fn wall_kick_r() {
        let mut g = Game::new();
        g.shape_controller.set_shape(Shape::El);
        g.shape_controller.set_position(Point::new(8, 3));
        g.shape_controller.set_orientation(Orientation::Up);
        g.start();
        let b = &mut g.board;
        b.occupy(&g.shape_controller);      
        assert!(g.board.0[3][8]);
        assert!(g.board.0[4][8]);
        assert!(g.board.0[5][8]);
        assert!(g.board.0[3][9]);        

        g.rotate(Direction::Ccw);
        
        assert!(g.board.0[3][7]);
        assert!(g.board.0[3][8]);
        assert!(g.board.0[3][9]);
        assert!(g.board.0[4][9]);
    }

    fn assert_el_at(c: &ShapeController, b: &Board) {
        let p = &c.position();
        let o = &c.orientation();
        match o {
            Orientation::Up => {
                assert!(b.0[p.y][p.x]);
                assert!(b.0[p.y+1][p.x]);
                assert!(b.0[p.y+2][p.x]);
                assert!(b.0[p.y][p.x+1]);
            },
            Orientation::Left => {
                assert!(b.0[p.y][p.x]);
                assert!(b.0[p.y][p.x+1]);
                assert!(b.0[p.y][p.x+2]);
                assert!(b.0[p.y+1][p.x+2]);               
            },
            _ => assert!(false, "not testing that orientation yet!")
        }
    }
    #[test]
    fn internal_kick_r() {
        // set up the game, but some junk in the board
        // kick off the junk.
        let mut g = Game::new();
        let config = vec![
            vec![false, false, false, false, true],
            vec![false, false, false, false, true],
            vec![false, false, false, false, true],
            vec![false, false, false, false, true],
            vec![false, false, false, false, true],
            vec![false, false, false, false, true],
            vec![false, false, false, false, true],
            vec![false, false, false, false, true],
        ];
        g.setup_board(config, Point::new(0,0), false);
        g.shape_controller.set_shape(Shape::El);
        g.shape_controller.set_position(Point::new(2, 1));
        g.shape_controller.set_orientation(Orientation::Up);
        g.start();
        g.board.occupy(&g.shape_controller);
        assert_el_at(&g.shape_controller, &g.board);
        g.rotate(Direction::Ccw);
        assert_el_at(&g.shape_controller, &g.board);
        assert!(g.shape_controller().position().x == 1, "expected right kick");
    }

    #[test]
    fn t_spin() {
        let mut g = Game::new();
        let config = vec![
            vec![false, false, false, false, false, false, false, false, false, false],
            vec![false, false, false, false, false, false, false, false, false, false],
            vec![false, false, false, false, false, false, false, false, false, false],
            vec![false, false, false, false, false, false, false, false, false, false],
            vec![false, false, false, false, true, true,  false,  false, false, false],
            vec![false, false, false, false, true, true,  false,  false, false, true],
            vec![true,  true,  true,  false, true, true,  false, false, false, true],
            vec![true,  true,  true,  true,  true, true,  true,  false, true,  true],
        ];
        g.setup_board(config, Point::new(0,0), false);
        g.shape_controller.set_shape(Shape::Tee);
        g.shape_controller.set_orientation(Orientation::Right);
        g.shape_controller.set_position(Point::new(7,0));
        g.board.occupy(&g.shape_controller);
        g.start();
        g.rotate(Direction::Cw);
        for x in 0..10 {
            assert!(g.board.0[0][x], "expected whole line to be true!");
        }
        assert!(g.shape_controller.position().x == 6, "expected a kick!");
    }

    #[test]
    fn kick_up() {
        let mut g = Game::new();
        let config = vec![
            vec![false, false, false, false, false, false, false,  false, false, false],
            vec![false, false, false, false, false, false, false,  false, false, false],
            vec![false, false, false, false, false, false, false,  false, false, false],
            vec![true,  false, true,  false, false,  true,  true,   true,  true,  true],
            vec![true,  false, true,  false,  true,  true,  true,   true,  true,  true],
            vec![true,  false, true,  false,  true,  true,  true,   true,  true,  true],
            vec![true,  false, true,  false,  true,  true,  true,   true,  true,  true],
            vec![true,  false, true,  false,  true,  true,  true,   true,  true,  true],
        ];
        g.setup_board(config, Point::new(0,0), false);
        g.shape_controller.set_shape(Shape::Eye);
        g.shape_controller.set_orientation(Orientation::Up);
        g.shape_controller.set_position(Point::new(1, 2));
        g.board.occupy(&g.shape_controller);
        g.start();
        for y in 2..4 {
            assert!(g.board.0[y][1], "missing part of I shape");
        }
        g.rotate(Direction::Ccw);
        for y in 0..3 {
            assert!(!g.board.0[y][1], "found part of I when there shouldn't have been any!");
        }
        for x in 1..5 {
            assert!(g.board.0[5][x], "missing part of I shape after rotation");
        }
    }
}