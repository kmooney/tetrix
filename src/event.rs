use crate::board::Board;
use crate::shape::Shape;
use crate::shape::Orientation;

#[derive(Clone, PartialEq, Debug)]
pub enum Output {
    GameOver,
    GameStarted,
    GameRunning,
    ScoreChanged,
    BoardUpdate(Board),
    HeldShape(Shape),
    RestoredShape(Shape),
    NextShape(Shape),
    RotatedShape(Orientation),
    MovedShape,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Input {
    StartGame,
    TickGame,
    Left,
    Right,
    Down,
    Drop,
    Hold,
    RestoreHold,
    Cw,
    Ccw,
}
use rand::Rng;

impl Input {
    pub fn rand_control() -> Input {
        let mut rng = rand::thread_rng();
        match rng.gen_range(2, 6) {
            2 => Input::Left,
            3 => Input::Right,
            4 => Input::Ccw,
            5 => Input::Cw,
            6 => Input::Hold,
            7 => Input::Drop,
            8 => Input::Down,
            _ => {Input::Ccw},
        }
    }
}