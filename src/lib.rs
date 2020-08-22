mod shape;
mod shape_state;
mod board;
mod event;
use board::Board;
use shape_state::{ShapeState, Direction};
use shape::{Shape, Point};

use std::sync::mpsc::{Sender, Receiver}; 

use event::{Input, Output};
const VERSION: f32 = 0.01;
const WIDTH: usize  = 10;
const HEIGHT: usize = 25;

#[derive(Debug, PartialEq)]
pub enum GameState {New, Playing, Over}

pub struct Game {
    score: u32,
    shape_controller: ShapeState,
    next_shape: Shape,
    hold_shape: Option<Shape>,
    state: GameState,
    pub board: Board,
    down_ready: bool,
    tx: Sender<Output>
}

impl Game {
    pub fn new(tx : Sender<Output>) -> Game {
        Game {
            score: 0,
            shape_controller: ShapeState::new(),
            next_shape: Shape::random(),
            hold_shape: None,
            state: GameState::New,
            board: Board::new(),
            down_ready: false,
            tx: tx  
      } 
    }

    pub fn shape_controller(&mut self) -> &mut ShapeState {
        return &mut self.shape_controller;
    }

    pub fn get_shape_controller(&self) -> &ShapeState {
        return &self.shape_controller
    }

    pub fn report(&self) -> String {
        let board = self.board;
        let current_piece_status = format!("{:?}", self.get_shape_controller().position());
        let current_piece_orientation = format!("shape = {:?}, orientation = {:?}", self.shape_controller.shape(), self.shape_controller.orientation());
        return String::from(format!("T E T R I X version {}\n{}\n{}\n{}\nscore: {}\nstate:{:?}\n", VERSION, current_piece_status, current_piece_orientation, board.report(), self.score, self.state))
    }

    fn check_collision(&self, s: &Shape, p: &Point) -> bool {
        let m = s.to_mat(&self.get_shape_controller().orientation());
        let b = self.board;
        for y in 0..4 {
            for x in 0..4 {
                let cell = m[3-y][x];
                if cell && (x + p.x >= WIDTH) {
                    return true;
                }
                if cell && b.0[y + p.y - 1][x + p.x] {
                    return true;
                }
            }
        }
        return false;
    }

    pub fn shape_collides(&self) -> bool {
        let s = &self.get_shape_controller().shape();
        let p = &self.get_shape_controller().position();
        if p.y == 0 {
            return true;
        }
        return self.check_collision(s, p);
    }

    pub fn check_game_over(&self) -> bool {
        let s = &self.get_shape_controller().shape();
        let p = &self.get_shape_controller().position();
        return p.y >= 20 && self.check_collision(s, p);
    }

    pub fn rotate(&mut self, direction: Direction) {
        let c = &mut self.shape_controller;
        c.rotate(direction, &self.board);
    }

    fn action(&mut self, i: Input) {
        match i {
            Input::Left => self.shape_controller.left(&self.board),
            Input::Right => self.shape_controller.right(&self.board),
            Input::Drop => self.shape_controller.drop(&self.board),
            Input::Down => {},
            Input::Hold => {},
            Input::Cw => self.shape_controller.rotate(Direction::Cw, &self.board),
            Input::Ccw => self.shape_controller.rotate(Direction::Ccw, &self.board),
            Input::TickGame => {self.down_ready = true;},
            _ => {}
        }
    }

    pub fn next(&mut self, i: Input) {
        match self.state { 
            GameState::Playing => {},
            _ => return,
        }
        self.board.vacate(
            &self.shape_controller.shape().to_mat(self.shape_controller.orientation()),
            self.shape_controller.position()
        );
        
        self.action(i);
        
        if self.shape_collides() {
            if self.check_game_over() {
                self.state = GameState::Over;
                self.tx.send(Output::GameOver).unwrap();
            }
            self.board.occupy(
                &self.shape_controller.shape().to_mat(self.shape_controller.orientation()),
                self.shape_controller.position()
            );
            self.shape_controller = ShapeState::new();
        } else {
            if self.down_ready {
                self.shape_controller.down();
                self.down_ready = false;
            }
            self.board.occupy(
                &self.shape_controller.shape().to_mat(self.shape_controller.orientation()),
                self.shape_controller.position()
            );
        }
        self.tx.send(Output::BoardUpdate(self.board)).unwrap();
    }

    pub fn start(&mut self) {
        self.state = GameState::Playing;
        self.tx.send(Output::GameStarted).unwrap();
    }

    pub fn clear_lines(&mut self) {
        let mut clear_count = 0;
        let mut y = 0;
        
        'outer: while y < HEIGHT {        
            for x in 0..WIDTH {
                if !self.board.0[y][x] {
                    y += 1;
                    continue 'outer;
                }
            }
            'fall: for z in y..HEIGHT - 1 {
                let mut empty_line = true;
                for x in 0..WIDTH {
                    if self.board.0[z+1][x] {
                        empty_line = false;
                    }
                    self.board.0[z][x] = self.board.0[z+1][x];
                }
                if empty_line {
                    break 'fall;
                }
            }
            clear_count += 1;
        }
        self.score += clear_count;
    }

}

use std::thread;
use std::sync::mpsc::channel;

pub fn game() -> (thread::JoinHandle<GameState>, Receiver<Output>, Sender<Input>) {
    let (txo, rxo) = channel();
    let (txi, rxi) = channel();

    let h = thread::spawn(move|| {
        let mut g = Game::new(txo);
        while g.state != GameState::Over {
            let mut check_messages = true;
            while check_messages {
                match rxi.try_recv() {
                    Ok(r) => {
                        match r {
                            Input::StartGame => {
                                g.start();
                            },
                            _ => {
                                g.next(r);
                            }
                        }
                    },
                    Err(_) => {
                        check_messages = false;
                    }
                };
            }
        }
        g.state
    });
    return (h, rxo, txi);
}


// tests start here!!


#[cfg(test)]
mod tests {
    use std::time;
    use crate::shape::Orientation;
    use super::*;

    #[test]
    fn game() {
        // when the game starts, there should be a shape controller with the current shape
        // and there should be a next shape.  
        // there should be no "hold" shape
        let (tx, _rx) = channel();
        let g = Game::new(tx);

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
        let mut board = g.board;
        board.setup(config, Point{x: 1, y: 3}, true);
        board.reset();
        let mut trues = 0;
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                match board.0[y][x] {
                    true => trues += 1,
                    false => {}
                }
            }
        }
        assert_eq!(trues, 0, "no boxes after board reset!");
    }

    #[test]
    fn rotate() {
        let (tx, _rx) = channel();

        let mut g = Game::new(tx);
        let mut b = g.board;
        
        g.shape_controller().set_shape(Shape::El);
        g.shape_controller().set_position(Point::new(3,3));
        g.start();
        g.rotate(Direction::Ccw);
        {
            let mat = g.shape_controller.shape().to_mat(g.shape_controller.orientation());
            let pos = g.shape_controller.position();
            b.occupy(&mat, pos);
        }
        assert!(b.0[3][3]);
        assert!(b.0[3][4]);
        assert!(b.0[3][5]);
        assert!(b.0[4][5]);
    }

    #[test]
    fn wall_kick_l() {
        let (tx, _rx) = channel();

        let mut g = Game::new(tx);
        g.shape_controller().set_shape(Shape::El);
        g.shape_controller().set_position(Point::new(0, 3));
        g.start();
        g.rotate(Direction::Ccw);
        assert!(g.shape_controller().position().x == 0, "shape controller position should be at 0");
    }

    #[test]
    fn flush_wall_r() {
        let (tx, _rx) = channel();

        let mut g = Game::new(tx);
        g.shape_controller.set_shape(Shape::El);
        g.shape_controller.set_position(Point::new(8, 3));
        g.shape_controller.set_orientation(Orientation::Up);
        g.start();
        let mut b = g.board;
        b.occupy(
            &g.shape_controller.shape().to_mat(g.shape_controller.orientation()),
            g.shape_controller.position()
        );
        assert!(b.0[3][8], "box 1 in the wrong spot!");
        assert!(b.0[4][8]);
        assert!(b.0[5][8]);
        assert!(b.0[3][9]);
    }

    #[test]
    fn wall_kick_r() {
        let (tx, _rx) = channel();

        let mut g = Game::new(tx);
        g.shape_controller.set_shape(Shape::El);
        g.shape_controller.set_position(Point::new(8, 3));
        g.shape_controller.set_orientation(Orientation::Up);
        g.start();
        let mut b = g.board;
        b.occupy(
            &g.shape_controller.shape().to_mat(g.shape_controller.orientation()),
            g.shape_controller.position()
        );      
        assert!(b.0[3][8]);
        assert!(b.0[4][8]);
        assert!(b.0[5][8]);
        assert!(b.0[3][9]);        

        g.rotate(Direction::Ccw);
        
        assert!(g.shape_controller().position().x == 7, "expected kick on shape");
    }

  
    #[test]
    fn internal_kick_r() {
        // set up the game, put some junk in the board
        // kick off the junk.
        let (tx, _rx) = channel();

        let mut g = Game::new(tx);
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
        g.board.setup(config, Point::new(0,0), false);
        g.shape_controller.set_shape(Shape::El);
        g.shape_controller.set_position(Point::new(2, 1));
        g.shape_controller.set_orientation(Orientation::Up);
        g.start();

        g.rotate(Direction::Ccw);

        assert!(g.shape_controller().position().x == 1, "expected right kick");
    }

    #[test]
    fn t_spin() {
        let (tx, _rx) = channel();

        let mut g = Game::new(tx);
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
        g.board.setup(config, Point::new(0,0), false);
        g.shape_controller.set_shape(Shape::Tee);
        g.shape_controller.set_orientation(Orientation::Right);
        g.shape_controller.set_position(Point::new(7,0));
        g.start();
        g.rotate(Direction::Cw);        
        assert!(g.shape_controller.position().x == 6, "expected a kick!");
    }

    #[test]
    fn kick_up() {
        let (tx, _rx) = channel();

        let mut g = Game::new(tx);
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
        g.board.setup(config, Point::new(0,0), false);
        g.shape_controller.set_shape(Shape::Eye);
        g.shape_controller.set_orientation(Orientation::Up);
        g.shape_controller.set_position(Point::new(1, 2));
        g.start();
        
        g.rotate(Direction::Ccw);
        
        assert!(5 == g.shape_controller().position().y, "position should be 5,1 but was {:?}", g.shape_controller().position());
        assert!(1 == g.shape_controller().position().x, "position should be 5,1 but was {:?}", g.shape_controller().position());

    }

    #[test]
    fn clear_lines() {
        let (tx, _rx) = channel();

        let mut g = Game::new(tx);
        let config = vec![
            vec![false, false, false, false, false, false, false,  false, false, false],
            vec![false, false, false, false, false, false, false,  false, false, false],
            vec![false, false, false, false, false, false, false,  false, false, false], 
            vec![true,  false, true,   true,  true,  true,  true,   true,  true,  true],
            vec![true,  false, true,   true,  true,  true,  true,   true,  true,  true],
            vec![true,  false, true,   true,  true,  true,  true,   true,  true,  true],
            vec![true,  false, true,   true,  true,  true,  true,   true,  true,  true],
        ];
        g.board.setup(config, Point::new(0,0), false);
        g.shape_controller.set_shape(Shape::Eye);
        g.shape_controller.set_orientation(Orientation::Up);
        g.shape_controller.set_position(Point::new(1,0));
        g.board.occupy(
            &g.shape_controller.shape().to_mat(g.shape_controller.orientation()),
            g.shape_controller.position()
        );
        g.start();
     
        assert!(g.score == 0, "line count should be zero");
        println!("{}", g.board.report());
        g.clear_lines();
        assert!(g.score == 4, "line count should be four");
        for y in 0..HEIGHT { 
            for x in 0..WIDTH { 
                assert!(!g.board.0[y][x], "board should be clear");
            }
        }
    }

    #[test]
    fn drop() {
        let (tx, _rx) = channel();

        let mut g = Game::new(tx);
        let mut b = Board::new();
        let config = vec![
            vec![false, false, false, false, false, false, false,  false, false, false],
            vec![false, false, false, false, false, false, false,  false, false, false],
            vec![false, false, false, false, false, false, false,  false, false, false], 
            vec![true,  false, true,   true,  true,  true,  true,   true,  true,  true],
            vec![true,  false, true,   true,  true,  true,  true,   true,  true,  true],
            vec![true,  false, true,   true,  true,  true,  true,   true,  true,  true],
            vec![true,  false, true,   true,  true,  true,  true,   true,  true,  true],
        ];
        b.setup(config, Point::new(0,0), false);
        g.shape_controller.set_shape(Shape::El);
        g.shape_controller.set_orientation(Orientation::Down);
        g.shape_controller.set_position(Point::new(0,0));
        g.start();
        g.shape_controller().drop(&b);
        assert!(g.shape_controller().position().x == 0, "x should be 0");
        assert!(g.shape_controller().position().y == 1, "y should be 1");
    }

    #[test]
    fn play() {
        let (h, _rx, txi) = crate::game();
        txi.send(Input::StartGame).unwrap();
        let txclock = txi.clone();
        thread::spawn(move || {
            while !txclock.send(Input::TickGame).is_err() {
                thread::sleep(time::Duration::from_millis(1));
            }
        });
        let v = h.join().unwrap();
        assert!(v == GameState::Over, "Game should be over but was {:?}", v);
    }

    #[test]
    fn read_events() {
        let (h, rx, txi) = crate::game();
        txi.send(Input::StartGame).unwrap();
        match rx.recv() {
            Ok(evt) => {
                assert!(evt == Output::GameStarted, "First event should be game start.  Got {:?} instead", evt);
            },
            Err(_) => {
                assert!(false, "Should have got event start; got error instead");
            } 
        };
        let txclock = txi.clone();
        thread::spawn(move || {
            while !txclock.send(Input::TickGame).is_err() {
                thread::sleep(time::Duration::from_millis(1));
            }
        });

        while let Ok(rmsg) = rx.recv() {
            match rmsg {
                Output::BoardUpdate(b) => {print!("{}", b.report())},
                _ => {println!("got some other message!")}
            }
        }

        let v = h.join().unwrap();
        assert!(v == GameState::Over, "Game should be over but was {:?}", v);
    }

    #[test]
    fn write_events() {
        let (h, rx, tx) = crate::game();
        tx.send(Input::StartGame).unwrap();
        match rx.recv() {
            Ok(evt) => {
                assert!(evt == Output::GameStarted, "event should have been game start")
            },
            Err(_) => {
                assert!(false, "there was an error after game start")
            }
        }

        let txclock = tx.clone();
        let txctrl = tx.clone();
        thread::spawn(move || {
            while !txclock.send(Input::TickGame).is_err() {
                thread::sleep(time::Duration::from_millis(5));
            }
        });
        thread::spawn(move || {
            while !txctrl.send(Input::rand_control()).is_err() {
                thread::sleep(time::Duration::from_millis(1));
            }
        });

        while let Ok(rmsg) = rx.recv() {
            match rmsg {
                Output::BoardUpdate(b) => {print!("{}", b.report())},
                _ => {println!("got some other message!")}
            }
        }


        h.join().unwrap();
    }

}