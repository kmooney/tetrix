pub mod shape;
mod shape_state;
pub mod event;
pub mod board;
use board::Board;
use shape_state::{ShapeState, Direction};
use shape::{Shape, Point};
use std::collections::VecDeque;

use std::time;

use std::sync::{Arc, Mutex, RwLock};
use std::vec::Vec;
use std::collections::HashMap;

use uuid::Uuid;

use std::sync::mpsc::{Sender, Receiver}; 

use event::{Input, Output};
const VERSION: f32 = 0.01;
pub const WIDTH: usize  = 10;
pub const HEIGHT: usize = 25;

use log;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum GameState {New, Playing, Over}

pub struct Game {
    score: u32,
    shape_controller: ShapeState,
    next_shape: Shape,
    hold_shape: Option<Shape>,
    state: GameState,
    pub board: Board,
    double_down: bool,
    down_ready: bool,
    tx: Sender<Output>,
    input_buffer: VecDeque<Input>,
    hold_allowed: bool,
    did_hold: bool
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
            double_down: false,
            down_ready: false,
            tx: tx,
            input_buffer: VecDeque::new(),
            hold_allowed: true,
            did_hold: false
      } 
    }

    pub fn shape_controller(&mut self) -> &mut ShapeState {
        return &mut self.shape_controller
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
        let m = s.to_mat(self.get_shape_controller().orientation());
        let b = self.board;
        for y in 0..4 {
            for x in 0..4 {
                let cell = m[3-y][x];
                if cell != None && (x + p.x >= WIDTH) {
                    return true;
                }
                if cell != None && b.0[y + p.y - 1][x + p.x] != None {
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
        self.tx.send(Output::RotatedShape(c.orientation())).unwrap();
    }

    fn action(&mut self, i: Input) {
        self.double_down = false;
        match i {
            Input::Left => match self.shape_controller.left(&self.board) {
                true => self.tx.send(Output::MovedShape).unwrap(),
                false => {}
            },
            Input::Right => match self.shape_controller.right(&self.board) {
                true => self.tx.send(Output::MovedShape).unwrap(),
                false => {}
            },
            Input::Drop => self.shape_controller.drop(&self.board),
            Input::Down => {self.double_down = true},
            Input::Hold => {
                /*
                    If there is a hold shape, pressing "hold" should switch the current shape
                    and the hold shape. The next shape stays the same.

                    If there is NOT a hold shape, pressing "hold" should make current shape the hold shape 
                    and make the "next" shape the current shape. The "next next" is randomly generated.
                */
                if self.hold_allowed {
                    match self.hold_shape {
                        Some(shape) => {
                            self.hold_shape = Some(self.shape_controller.shape());
                            self.shape_controller = ShapeState::new_from_shape(shape);
                        },
                        None => {
                            self.hold_shape = Some(self.shape_controller.shape());
                            self.shape_controller = ShapeState::new_from_shape(self.next_shape);
                            self.next_shape = Shape::random();
                            self.tx.send(Output::NextShape(self.next_shape)).unwrap();
                        }
                    }                
                    self.tx.send(Output::HeldShape(self.hold_shape.unwrap())).unwrap();
                    self.hold_allowed = false;
                    self.did_hold = true;
                }
            },
            Input::Cw => self.shape_controller.rotate(Direction::Cw, &self.board),
            Input::Ccw => self.shape_controller.rotate(Direction::Ccw, &self.board),
            Input::TickGame => {self.down_ready = true;},
            _ => {}
        }
    }

    pub fn next(&mut self, i: Input) {
        
        self.input_buffer.push_front(i.clone());
        if self.input_buffer.len() > 3 {
            self.input_buffer.pop_back();
        }

        match self.state { 
            GameState::Playing => {},
            _ => return,
        }

        let from_point = self.shape_controller.position().clone();
        let from_orientation = self.shape_controller.orientation().clone();

        self.board.vacate(
            &self.shape_controller.shape().to_mat(self.shape_controller.orientation()),
            self.shape_controller.position()
        );
      
        self.action(i);

        let count = match self.double_down {
            true => 2,
            false => 1
        };

        for _i in 0..count {
            if self.shape_collides() {
                let to_point = self.shape_controller.position().clone();
                if self.check_game_over() {
                    self.state = GameState::Over;
                    self.tx.send(Output::GameOver).unwrap();
                }
                self.board.occupy(
                    &self.shape_controller.shape().to_mat(self.shape_controller.orientation()),
                    self.shape_controller.position()
                );
                // this would be the last gasp of the shape before it locks..

                // if the last Input we got was a Tick, the lock it down - otherwise
                // "continue" the loop and await more input????
                // aftertouch code is here...
                self.tx.send(Output::ShapePosition(self.shape_controller.shape(), Some(from_orientation), self.shape_controller.orientation(), Some(from_point), to_point)).unwrap();                
                
                if self.input_buffer[0] == Input::Drop || (self.down_ready && self.input_buffer.len() > 1 && self.input_buffer[0] == Input::TickGame && self.input_buffer[1] == Input::TickGame) {
                    
                    self.tx.send(Output::ShapeLocked(self.shape_controller.shape(), self.board)).unwrap();
                    self.hold_allowed = true;
                    
                    self.shape_controller = ShapeState::new_from_shape(self.next_shape);                
                    self.next_shape = Shape::random();                                
                    self.tx.send(Output::NextShape(self.next_shape)).unwrap();

                    let to_point = self.shape_controller.position().clone();
                    // this is the new shape
                    self.tx.send(Output::ShapePosition(self.shape_controller.shape(), None, self.shape_controller.orientation(), None, to_point)).unwrap();        
                    self.clear_lines(); 
                }
            } else {
                if self.down_ready {
                    match self.shape_controller.down() {
                        true => self.tx.send(Output::MovedShape).unwrap(),
                        false => {}
                    }
                    self.down_ready = false;
                }
                let to_point = self.shape_controller.position().clone();
                if self.did_hold {
                    self.tx.send(Output::ShapePosition(self.shape_controller.shape(), None, self.shape_controller.orientation(), None, to_point)).unwrap();
                    self.did_hold = false;
                } else {
                    self.tx.send(Output::ShapePosition(self.shape_controller.shape(), Some(from_orientation), self.shape_controller.orientation(), Some(from_point), to_point)).unwrap();        
                }
                self.board.occupy(
                    &self.shape_controller.shape().to_mat(self.shape_controller.orientation()),
                    self.shape_controller.position()
                );
            }
        }
        self.tx.send(Output::BoardUpdate(self.board)).unwrap();
    }

    pub fn start(&mut self) {
        self.state = GameState::Playing;
        self.tx.send(Output::GameStarted).unwrap();
        self.tx.send(Output::NextShape(self.next_shape)).unwrap();
    }

    pub fn quit(&mut self) {
        self.state = GameState::Over;
        self.tx.send(Output::GameOver).unwrap();
    }

    pub fn clear_lines(&mut self) {
        let mut clear_count : u8 = 0;
        let mut y = 0;
        
        'outer: while y < HEIGHT {        
            for x in 0..WIDTH {
                if self.board.0[y][x] == None{
                    y += 1;
                    continue 'outer;
                }
            }
            'fall: for z in y..HEIGHT - 1 {
                let mut empty_line = true;
                for x in 0..WIDTH {
                    if self.board.0[z+1][x] != None {
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
        // TODO: more complex score calculation
        self.score += clear_count as u32;
        if clear_count != 0 {
            self.tx.send(Output::ScoreUpdate(self.score)).unwrap();
            self.tx.send(Output::LineCompleted(clear_count, self.board)).unwrap();
        }
    }

}

use std::thread;
use std::sync::mpsc::channel;


pub struct GameHandle {
    join_handle: thread::JoinHandle<GameState>,
    output_receiver: Arc<Mutex<Receiver<Output>>>,
    input_sender: Arc<Mutex<Sender<Input>>>
}

impl GameHandle {
    pub fn tuple(&self) -> (&thread::JoinHandle<GameState>, Arc<Mutex<Receiver<Output>>>, Arc<Mutex<Sender<Input>>>) {
        (&self.join_handle, self.output_receiver.clone(), self.input_sender.clone())
    }
}

pub fn game() -> GameHandle {
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
                            Input::EndGame => {
                                g.quit();
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
    return GameHandle{join_handle: h, output_receiver: Arc::new(Mutex::new(rxo)), input_sender: Arc::new(Mutex::new(txi))};
}

pub struct GameWrapper {
    h: GameHandle,
    ob: Arc<Mutex<VecDeque<Output>>>,
    level: Arc<RwLock<u8>>
}

impl GameWrapper {

    pub fn new(h: GameHandle) -> GameWrapper {
        log::debug!("Creating new GameWrapper!");
        let ob = Arc::new(Mutex::new(VecDeque::new()));
        let q = ob.clone();
        let rxo = h.output_receiver.clone();
        let lvl: Arc<RwLock<u8>> = Arc::new(RwLock::new(1));

        thread::spawn(move || {
            let mut done = false;
            let rxo = rxo.lock().unwrap();
            while !done {
                match rxo.recv() {
                    Ok(evt) => {
                        let mut q = q.lock().unwrap(); 
                        q.push_back(evt);
                    },
                    Err(_) => {done = true;}
                }
            }

        });
        let txclock = h.input_sender.clone();
        let clocklvl = lvl.clone();
        thread::spawn(move || {    
            // i *think* this lock is released after we send and check error
            // so it should be unlocked most of the time.        
            while !txclock.lock().unwrap().send(Input::TickGame).is_err() {
                log::debug!("Going to tick the game");
                // level 10 = 100
                // level 1 = 1000
                // 1000 - (level) * 100
                let t_lvl : u8;
                {
                    t_lvl = *clocklvl.read().unwrap();
                }
                let sleep_time = 1000 - t_lvl as u64 * 100;
                log::debug!("sleeping for {:?}", sleep_time);
                thread::sleep(time::Duration::from_millis(sleep_time));
            }
        });
        return GameWrapper {h: h, ob: ob, level: lvl};
    }

    pub fn drain(ob : Arc<Mutex<VecDeque<Output>>>) -> Vec<Output> {
        let mut v = Vec::new();
        {
            let mut q = ob.lock().unwrap();
            // a bit gross because we block the queue for the whole
            // read. rwblock wouldn't be better since we modify the 
            // the queue right after we read.... 
            while !q.is_empty() {
                v.push(q.pop_front().unwrap());
            }
        }
        return v;   
    }

    pub fn queue(&self) -> Arc<Mutex<VecDeque<Output>>> {
        return self.ob.clone();
    }

    pub fn set_level(&self, lvl: u8) {
        let mut l = self.level.write().unwrap();
        *l = lvl;
    }

    pub fn send(&self, input: Input) {
        match self.h.input_sender.lock().unwrap().send(input) {
            Err(e) => println!("ERROR {}", e),
            _ => {}
        }
    }
}


pub struct GameMaster{
    pool: Arc<RwLock<HashMap<Uuid, Arc<GameWrapper>>>>
}

impl GameMaster {
    pub fn new() -> GameMaster {
        let v : HashMap<Uuid, Arc<GameWrapper>> = HashMap::new();
        return GameMaster{pool: Arc::new(RwLock::new(v))};
    }

    pub fn count(&self) -> usize {
        return self.pool.read().unwrap().len();
    }

    pub fn new_game(&self) -> Uuid {
        // check for max games and don't allow us to make
        // more than that many.
        let uuid = Uuid::new_v4();
        let mut mut_pool = self.pool.write().unwrap();
        mut_pool.insert(uuid, Arc::new(GameWrapper::new(game())));
        return uuid;
    }

    pub fn game(&self, u: Uuid) -> Option<Arc<GameWrapper>> {
        let pool = self.pool.read().unwrap();
        if pool.contains_key(&u) {            
            return Some(pool[&u].clone());
        }
        return None;
    }
}


// tests start here


#[cfg(test)]
mod tests {
    use crate::shape::Orientation;
    use super::*;

    #[test]
    fn gw() {
        let gw = GameWrapper::new(crate::game());

        assert_eq!(GameWrapper::drain(gw.queue()).len(), 0, "zero messages before start");
    }

    #[test]
    fn gw_buffer() {
        let gw = GameWrapper::new(crate::game());
        assert_eq!(GameWrapper::drain(gw.queue()).len(), 0, "zero messages before start");
        gw.send(Input::StartGame);
        std::thread::sleep(time::Duration::from_millis(100));
        assert!(GameWrapper::drain(gw.queue()).len() > 0, "should have buffered some output by now");
    }   

    #[test] 
    fn gm() {
        let gm = GameMaster::new();       
        assert_eq!(gm.count(), 0, "ran");
        
    }

    #[test]
    fn gm_new_game() {
        let gm = GameMaster::new();       
        assert_eq!(gm.count(), 0, "ran");
        gm.new_game();
        assert_eq!(gm.count(), 1, "new game");
    }

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
            vec![None, Some(Shape::random()), None, Some(Shape::random()), None, None, Some(Shape::random())],
            vec![None, Some(Shape::random()), None, Some(Shape::random()), None, None, Some(Shape::random())],
            vec![None, Some(Shape::random()), None, Some(Shape::random()), None, None, Some(Shape::random())],
            vec![None, Some(Shape::random()),  Some(Shape::random()), Some(Shape::random()), None, None, Some(Shape::random())],
            vec![None, Some(Shape::random()), None, Some(Shape::random()), None, None, Some(Shape::random())],
            vec![None, Some(Shape::random()), None, Some(Shape::random()), None, None, Some(Shape::random())],
            vec![None, Some(Shape::random()), None, Some(Shape::random()), None, None, Some(Shape::random())],
        ];
        let mut board = g.board;
        board.setup(config, Point{x: 1, y: 3}, true);
        board.reset();
        let mut trues = 0;
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                match board.0[y][x] {
                    Some(_) => trues += 1,
                    None => {}
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
        assert!(b.0[3][3] != None);
        assert!(b.0[3][4] != None);
        assert!(b.0[3][5] != None);
        assert!(b.0[4][5] != None);
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
        assert!(b.0[3][8] != None, "box 1 in the wrong spot!");
        assert!(b.0[4][8] != None);
        assert!(b.0[5][8] != None);
        assert!(b.0[3][9] != None);
    }

    #[test]
    fn flush_wall_r2() {
        let (tx, _rx) = channel();
        let mut g = Game::new(tx);
        g.shape_controller.set_shape(Shape::Eye);
        g.shape_controller.set_position(Point::new(5, 3));
        g.shape_controller.set_orientation(Orientation::Up);
        g.start();
        let mut b = g.board;
        b.occupy(
            &g.shape_controller.shape().to_mat(g.shape_controller.orientation()),
            g.shape_controller.position()
        );
        println!("{}",b.report());
        assert!(b.0[3][5] != None);
        assert!(b.0[4][5] != None);
        assert!(b.0[5][5] != None);
        assert!(b.0[6][5] != None);

        
        b.vacate(
            &g.shape_controller.shape().to_mat(g.shape_controller.orientation()),
            g.shape_controller.position()
        );
        
        g.shape_controller.right(&g.board);
        
        b.occupy(
            &g.shape_controller.shape().to_mat(g.shape_controller.orientation()),
            g.shape_controller.position()
        );

        println!("{}",b.report());
        assert!(b.0[3][6] != None, "not at y = 6");
        assert!(b.0[4][6] != None);
        assert!(b.0[5][6] != None);
        assert!(b.0[6][6] != None);

         
        b.vacate(
            &g.shape_controller.shape().to_mat(g.shape_controller.orientation()),
            g.shape_controller.position()
        );
        
        g.shape_controller.right(&g.board);
        
        b.occupy(
            &g.shape_controller.shape().to_mat(g.shape_controller.orientation()),
            g.shape_controller.position()
        );

        println!("{}",b.report());
        assert!(b.0[3][7] != None, "not at y = 7");
        assert!(b.0[4][7] != None);
        assert!(b.0[5][7] != None);
        assert!(b.0[6][7] != None);


        b.vacate(
            &g.shape_controller.shape().to_mat(g.shape_controller.orientation()),
            g.shape_controller.position()
        );
        
        g.shape_controller.right(&g.board);
        
        b.occupy(
            &g.shape_controller.shape().to_mat(g.shape_controller.orientation()),
            g.shape_controller.position()
        );

        println!("{}",b.report());
        assert!(b.0[3][8] != None, "not at y = 8");
        assert!(b.0[4][8] != None);
        assert!(b.0[5][8] != None);
        assert!(b.0[6][8] != None);

        
        b.vacate(
            &g.shape_controller.shape().to_mat(g.shape_controller.orientation()),
            g.shape_controller.position()
        );
        
        g.shape_controller.right(&g.board);
        
        b.occupy(
            &g.shape_controller.shape().to_mat(g.shape_controller.orientation()),
            g.shape_controller.position()
        );

        println!("{}",b.report());
        assert!(b.0[3][9] != None, "not at y = 9");
        assert!(b.0[4][9] != None);
        assert!(b.0[5][9] != None);
        assert!(b.0[6][9] != None);


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
        assert!(b.0[3][8] != None);
        assert!(b.0[4][8] != None);
        assert!(b.0[5][8] != None);
        assert!(b.0[3][9] != None);        

        g.rotate(Direction::Ccw);
        
        assert!(g.shape_controller().position().x == 7, "expected kick on shape");
    }

    #[test]
    fn wall_kick_eye_r() {
        let (tx, _rx) = channel();

        let mut g = Game::new(tx);
        g.shape_controller.set_shape(Shape::Eye);
        g.shape_controller.set_position(Point::new(9, 3));
        g.shape_controller.set_orientation(Orientation::Up);
        g.start();
        let mut b = g.board;
        b.occupy(
            &g.shape_controller.shape().to_mat(g.shape_controller.orientation()),
            g.shape_controller.position()
        );

        assert!(b.0[3][9] != None);
        assert!(b.0[4][9] != None);
        assert!(b.0[5][9] != None);
        assert!(b.0[6][9] != None);        

        g.rotate(Direction::Ccw);
        
        assert!(g.shape_controller().position().x == 6, "expected kick on wall");    
    }

    #[test]
    fn shape_kick_eye_r() {

        let (tx, _rx) = channel();

        let mut g = Game::new(tx);
        g.shape_controller.set_shape(Shape::Eye);
        g.shape_controller.set_position(Point::new(7, 3));
        g.shape_controller.set_orientation(Orientation::Up);
        let config = vec![
            vec![None, None, None, None, None, None, None, None, None, Some(Shape::ElInv)],
            vec![None, None, None, None, None, None, None, None, None, Some(Shape::ElInv)],
            vec![None, None, None, None, None, None, None, None, None, Some(Shape::ElInv)],
            vec![None, None, None, None, None, None, None, None, Some(Shape::ElInv), Some(Shape::ElInv)],
            vec![None, None, None, None, None, None, None, None, None, Some(Shape::ElInv)],
            vec![None, None, None, None, None, None, None, None, None, Some(Shape::ElInv)],
            vec![None, None, None, None, None, None, None, None, Some(Shape::ElInv), Some(Shape::ElInv)],
            vec![None, None, None, None, None, None, None, None, None, Some(Shape::ElInv)],
            vec![None, None, None, None, None, None, None, None, None, Some(Shape::ElInv)],
            vec![None, None, None, None, None, None, None, None, Some(Shape::ElInv), Some(Shape::ElInv)],
        ];        
        g.board.setup(config, Point::new(0,0), false);
        g.start();
        let mut b = g.board;
        b.occupy(
            &g.shape_controller.shape().to_mat(g.shape_controller.orientation()),
            g.shape_controller.position()
        );
        println!("{}", b.report());
        assert!(b.0[3][7] != None);
        assert!(b.0[4][7] != None);
        assert!(b.0[5][7] != None);
        assert!(b.0[6][7] != None);

        b.vacate(
            &g.shape_controller.shape().to_mat(g.shape_controller.orientation()),
            g.shape_controller.position()
        );
        g.rotate(Direction::Ccw);
        
        assert!(g.shape_controller().position().x == 4, "expected kick on shape"); 
        b.occupy(
            &g.shape_controller.shape().to_mat(g.shape_controller.orientation()),
            g.shape_controller.position()
        );
        println!("{}", b.report());
        assert!(b.0[3][9].unwrap() == Shape::ElInv, "Should be ElInv! But was {:?}", b.0[3][9].unwrap());
        assert!(b.0[3][8].unwrap() == Shape::ElInv, "Should be ElInv! But was {:?}", b.0[3][8].unwrap());
        assert!(b.0[3][7].unwrap() == Shape::Eye, "Should be ElInv! But was {:?}", b.0[3][7].unwrap());
        
        

    }

    #[test]
    fn internal_kick_r() {
        // set up the game, put some junk in the board
        // kick off the junk.
        let (tx, _rx) = channel();

        let mut g = Game::new(tx);
        let config = vec![
            vec![None, None, None, None, Some(Shape::random())],
            vec![None, None, None, None, Some(Shape::random())],
            vec![None, None, None, None, Some(Shape::random())],
            vec![None, None, None, None, Some(Shape::random())],
            vec![None, None, None, None, Some(Shape::random())],
            vec![None, None, None, None, Some(Shape::random())],
            vec![None, None, None, None, Some(Shape::random())],
            vec![None, None, None, None, Some(Shape::random())],
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
            vec![None, None, None, None, None, None, None, None, None, None],
            vec![None, None, None, None, None, None, None, None, None, None],
            vec![None, None, None, None, None, None, None, None, None, None],
            vec![None, None, None, None, None, None, None, None, None, None],
            vec![None, None, None, None, Some(Shape::random()), Some(Shape::random()),  None,  None, None, None],
            vec![None, None, None, None, Some(Shape::random()), Some(Shape::random()),  None,  None, None, Some(Shape::random())],
            vec![Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),  None, Some(Shape::random()), Some(Shape::random()),  None, None, None, Some(Shape::random())],
            vec![Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()), Some(Shape::random()),  Some(Shape::random()),  None, Some(Shape::random()),  Some(Shape::random())],
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
            vec![None, None, None, None, None, None, None,  None, None, None],
            vec![None, None, None, None, None, None, None,  None, None, None],
            vec![None, None, None, None, None, None, None,  None, None, None],
            vec![Some(Shape::random()),  None, Some(Shape::random()),  None, None,  Some(Shape::random()),  Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random())],
            vec![Some(Shape::random()),  None, Some(Shape::random()),  None,  Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random())],
            vec![Some(Shape::random()),  None, Some(Shape::random()),  None,  Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random())],
            vec![Some(Shape::random()),  None, Some(Shape::random()),  None,  Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random())],
            vec![Some(Shape::random()),  None, Some(Shape::random()),  None,  Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random())],
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
            vec![None, None, None, None, None, None, None,  None, None, None],
            vec![None, None, None, None, None, None, None,  None, None, None],
            vec![None, None, None, None, None, None, None,  None, None, None], 
            vec![Some(Shape::random()),  None, Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random())],
            vec![Some(Shape::random()),  None, Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random())],
            vec![Some(Shape::random()),  None, Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random())],
            vec![Some(Shape::random()),  None, Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random())],
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
                assert!(g.board.0[y][x] == None, "board should be clear");
            }
        }
    }

    #[test]
    fn drop() {
        let (tx, _rx) = channel();

        let mut g = Game::new(tx);
        let mut b = Board::new();
        let config = vec![
            vec![None, None, None, None, None, None, None,  None, None, None],
            vec![None, None, None, None, None, None, None,  None, None, None],
            vec![None, None, None, None, None, None, None,  None, None, None], 
            vec![Some(Shape::random()),  None, Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random())],
            vec![Some(Shape::random()),  None, Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random())],
            vec![Some(Shape::random()),  None, Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random())],
            vec![Some(Shape::random()),  None, Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random())],
        ];
        b.setup(config, Point::new(0,0), false);
        g.shape_controller.set_shape(Shape::El);
        g.shape_controller.set_orientation(Orientation::Down);
        g.shape_controller.set_position(Point::new(0,10));
        g.start();
        g.shape_controller().drop(&b);
        println!("{}", b.report());
        assert!(g.shape_controller().position().x == 0, "x should be 0");
        assert!(g.shape_controller().position().y == 2, "y should be 2, but is {}", g.shape_controller().position().y);
    }

    
    #[test]
    fn drop_bug_one() {
        /*
        - drop scenario error: the L at 3,3 should be at 2,3
        05  xxx
        04 xxxx    x
        03 xxxx    xx
        02 xx x     x
        01 xxxxx   xx
        00 xx  xxx xx
        -------------
          |0123456789
        */

        let (tx, _rx) = channel();

        let mut g = Game::new(tx);
        let mut b = Board::new();
        let config = vec![
            vec![None, None, None, None, None, None, None,  None, None, None],
            vec![None, Some(Shape::Eye), None, None, None, None, None,  None, None, None],
            vec![Some(Shape::Eye), Some(Shape::Eye), None, None, None, None, None,  None, None, None], 
            vec![Some(Shape::Eye),  Some(Shape::Eye), None, None, Some(Shape::Zee), None, None, None, None, None],
            vec![Some(Shape::Eye),  Some(Shape::Eye), None, None,  Some(Shape::Zee),  Some(Shape::Zee),  None, None, None, None],
            vec![Some(Shape::Eye),  Some(Shape::Zee), Some(Shape::Zee),  None,  Some(Shape::ElInv),  Some(Shape::Zee),  None,   None,  Some(Shape::Square),  Some(Shape::Square)],
            vec![Some(Shape::Zee),  Some(Shape::Zee), None,  None, Some(Shape::ElInv),  Some(Shape::ElInv),  Some(Shape::ElInv),  None,  Some(Shape::Square),  Some(Shape::Square)],
        ];
        b.setup(config, Point::new(0,0), false);
        g.shape_controller.set_shape(Shape::El);
        g.shape_controller.set_orientation(Orientation::Down);
        g.shape_controller.set_position(Point::new(2,16));
        g.start();
        g.shape_controller().drop(&b);        
        b.occupy(
            &g.shape_controller.shape().to_mat(g.shape_controller.orientation()),
            g.shape_controller.position()
        );
        println!("{}", b.report());
        assert!(g.shape_controller().position().x == 2, "x should be 2");
        assert!(g.shape_controller().position().y == 0, "y should be 0 but was {}", g.shape_controller().position().y);
        assert!(b.0[0][3] != None, "Spot 3, 0 should be filled!");
        assert!(b.0[0][3].unwrap() == Shape::El, "The El shape should occupy x = 3, y = 0");
    }


    #[test]
    fn play() {
        let g = crate::game();
        let (_h, _rx, txi) = g.tuple();
        txi.lock().unwrap().send(Input::StartGame).unwrap();
        let txclock = txi.clone();
        thread::spawn(move || {
            while !txclock.lock().unwrap().send(Input::TickGame).is_err() {
                thread::sleep(time::Duration::from_millis(1));
            }
        });
        let v = g.join_handle.join().unwrap();
        assert!(v == GameState::Over, "Game should be over but was {:?}", v);
    }

    #[test]
    fn read_events() {
        let g = crate::game();
        let (_h, rx, txi) = g.tuple();
        txi.lock().unwrap().send(Input::StartGame).unwrap();
        match rx.lock().unwrap().recv() {
            Ok(evt) => {
                assert!(evt == Output::GameStarted, "First event should be game start.  Got {:?} instead", evt);
            },
            Err(_) => {
                assert!(false, "Should have got event start; got error instead");
            } 
        };
        let txclock = txi.clone();
        thread::spawn(move || {
            while !txclock.lock().unwrap().send(Input::TickGame).is_err() {
                thread::sleep(time::Duration::from_millis(1));
            }
        });

        while let Ok(rmsg) = rx.lock().unwrap().recv() {
            match rmsg {
                Output::BoardUpdate(b) => {print!("{}", b.report())},
                _ => {println!("got some other message!")}
            }
        }

        let v = g.join_handle.join().unwrap();
        assert!(v == GameState::Over, "Game should be over but was {:?}", v);
    }

    fn self_play(rx: &Receiver<Output>, tx: &Sender<Input>, no_input: bool, log: &mut std::vec::Vec<Output>) {
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
                thread::sleep(time::Duration::from_millis(10));
            }
        });

        if !no_input {
            thread::spawn(move || {
                while !txctrl.send(Input::rand_control()).is_err() {
                    thread::sleep(time::Duration::from_millis(70));
                }
            });
        }

        while let Ok(rmsg) = rx.recv() {
            log.push(rmsg.clone());
            match rmsg {
                Output::BoardUpdate(b) => {print!("{}", b.report())},
                x => {println!("got some other message: {:?}", x)}
            }
        }
    }

    #[test]
    fn write_events() {
        let g = crate::game();        
        let mut v = Vec::new();

        self_play(&g.output_receiver.lock().unwrap(), &g.input_sender.lock().unwrap(), false, &mut v);

        assert!(v.len() > 0, "expect the log to contain some output events");
        
        g.join_handle.join().unwrap();
    }

    #[test]
    fn trasheroonie() {
        let (_tx, _rx) = channel();
        let g = Game::new(_tx);
        let mut b = g.board;
        let mut trash_count = 0;
        b.trash(10);
        for x in b.0.iter() {
            for y in x.iter() {
                if *y != None {
                    trash_count += 1;
                }
            }
        }
        assert_eq!(trash_count, 10, "should be 10 trash, but there was {} trash", trash_count);

    }

    #[test]
    fn holds() {
        println!("running hold test");
        let g = crate::game();
        let (_h, rx, tx) = g.tuple();
        tx.lock().unwrap().send(Input::StartGame).unwrap();        
        match rx.lock().unwrap().recv() {
            Ok(_) => {
            },
            Err(_) => {
                assert!(false, "there was an error after game start")
            }
        }

        let txctrl = tx.clone();
        let txclock = tx.clone();

        thread::spawn(move || {
            while !txclock.lock().unwrap().send(Input::TickGame).is_err() {
                thread::sleep(time::Duration::from_millis(1));
            }
        });


        txctrl.lock().unwrap().send(Input::Hold).unwrap();
        let mut done = false;
        let mut counter = 0;
        while !done {
            println!("receiving..");
            match rx.lock().unwrap().recv() {
                Ok(response) => match response {
                    Output::HeldShape(shape) => {
                        println!("shape was {:?}", shape);
                        assert!(true, "we held the shape: {:?}", shape);
                        done = true
                    },
                    _ => {} 
                },
                Err(_) => {
                    assert!(false, "well, no way. we got an error response and that should be covered by another test.");
                }
            }
            counter = counter + 1;
            assert!(counter < 10, "we expected a response about holding the shape and did not get one :(");
        }
        
        g.join_handle.join().unwrap();
    }
 
    #[test]
    fn shape_lock() {
        let g = crate::game();
        let (_h, rx, tx) = g.tuple();
        let mut v = Vec::new();

        self_play(&rx.lock().unwrap(), &tx.lock().unwrap(), false, &mut v);

        assert!(v.len() > 0, "expect the log to contain some output events");

        let mut got_it = false;
        for o in v.iter() {
            match o {
                Output::ShapeLocked(_s, _b) => {got_it = true;},
                _ => {}
            }
        }

        assert!(got_it, "we shoulda got a ShapeLocked event");

        g.join_handle.join().unwrap();
    }

    #[test]
    fn lines_completed() {

        let (txo, rxo) = channel();
        let (txi, rxi) = channel();

        let mut g = Game::new(txo);
        let mut b = Board::new();
        let config = vec![
            vec![None, None, None, None, None, None, None,  None, None, None],
            vec![None, None, None, None, None, None, None,  None, None, None],
            vec![None, None, None, None, None, None, None,  None, None, None], 
            vec![Some(Shape::random()),  None, Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random())],
            vec![Some(Shape::random()),  None, Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random())],
            vec![Some(Shape::random()),  None, Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random())],
            vec![Some(Shape::random()),  None, Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random()),   Some(Shape::random()),  Some(Shape::random()),  Some(Shape::random())],
        ];
        let mut log = Vec::new();

        b.setup(config, Point::new(0,0), false);
        g.shape_controller.set_shape(Shape::El);
        g.shape_controller.set_orientation(Orientation::Down);
        g.shape_controller.set_position(Point::new(0,5));
        g.board = b;

        let h = thread::spawn(move|| {
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

        self_play(&rxo, &txi, true, &mut log);
        
        assert!(log.len() > 0, "expect the log to contain some output");
        let mut got_it = false;
        for o in log.iter() {
            match o {
                Output::LineCompleted(_n, _b) => {got_it = true;},
                _ => {}
            }
        }

        assert!(got_it, "we shoulda got a line completed message!");
        h.join().unwrap();
    }
}