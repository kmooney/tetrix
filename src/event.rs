#[derive(Clone, PartialEq, Debug)] 
pub enum Output {
    GameOver,
    GameStarted,
    GameRunning,
    ScoreChanged,
    BoardUpdate
}

#[derive(Clone, PartialEq, Debug)]
pub enum Input {
    // input events
    StartGame,
    TickGame,
    ActiveLeft,
    ActiveRight,
    ActiveQuickDrop
}