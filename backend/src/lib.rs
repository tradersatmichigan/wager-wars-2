use std::collections::BTreeMap;

struct Flip {
    probability: f32,
    payout: f32,
}

struct Bet {
    amount: u32,
    flips: Vec<u32>
}

struct Player {
    stack: u32,
    bet: Option<Bet>
}

enum RoundType {
    Joining,
    Betting,
    Results,
    Leaderboard,
}

pub struct Game {
    players: BTreeMap<String, Player>,
    current_rount: RoundType,
    flips: Vec<Vec<Flip>>,
    current_flip: usize,
}

impl Game {
    const INIT_STACK: u32 = 1000;

    pub fn new() -> Self {
        Self {
            players: BTreeMap::new(),
            current_rount: RoundType::Joining,
            flips: get_flips(),
            current_flip: 0
        }
    }

    pub fn add_player(&mut self) -> anyhow::Result<()> {
        todo!()
    }
}

fn get_flips() -> Vec<Vec<Flip>> {
    todo!()
}
