#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    MainMenu,
    LoadMenu,
    InGame,
    LoadGame,
    Paused,
}