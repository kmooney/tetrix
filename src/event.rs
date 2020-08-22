use crate::board::Board;

#[derive(Clone, PartialEq, Debug)]
pub enum Output {
    GameOver,
    GameStarted,
    GameRunning,
    ScoreChanged,
    BoardUpdate(Board)
}

#[derive(Clone, PartialEq, Debug)]
pub enum Input {
    StartGame,
    TickGame,
    ActiveLeft,
    ActiveRight,
    ActiveQuickDrop
}