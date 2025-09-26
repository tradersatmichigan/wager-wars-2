pub mod api {

    pub async fn login() {

    }

}

pub mod game {
    use std::collections::HashMap;

    use anyhow::Result;

    pub struct Game {
        players: HashMap<String, Player>,
        mode: Mode,
        flips: Vec<Vec<Flip>>,
        current_flip: usize,
    }

    impl Game {
        pub fn add_player(&mut self, name: String) -> Result<()> {
            if self.players.contains_key(&name) {
                anyhow::bail!("This username is already taken");
            }

            if self.mode != Mode::Joining {
                anyhow::bail!("This game is already in progress");
            }

            self.players.insert(name, Player::default());
            Ok(())
        }

        pub fn bet(&mut self, name: String, bet: Bet) -> Result<()> {
            if self.mode != Mode::Betting {
                anyhow::bail!("Betting is not open right now");
            }

            for idx in bet.flips.iter() {
                if *idx >= self.flips[self.current_flip].len() {
                    anyhow::bail!("Invalid flip index");
                }
            }

            let player = self.players.get_mut(&name).ok_or(anyhow::anyhow!("You must be signed in to play this game"))?;

            if player.stack < bet.amount {
                anyhow::bail!("You cannot bet more than your stack");
            }

            player.current_bet = Some(bet);
            Ok(())
        }

        pub fn tick(&mut self) {
            match self.mode {
                Mode::Joining => {
                    self.mode = Mode::Betting;
                    self.current_flip = 0;
                }
                Mode::Betting => {

                }
                Mode::BetResult => {

                }
                Mode::Done => {}
            }
        }
    }

    struct Player {
        stack: u64,
        current_bet: Option<Bet>,
    }

    impl Default for Player {
        fn default() -> Self {
            Self {
                stack: 10000,
                current_bet: None,
            }
        }
    }

    pub struct Bet {
        amount: u64,
        flips: Vec<usize>,
    }

    #[derive(PartialEq, Eq)]
    enum Mode {
        Joining,
        Betting,
        BetResult,
        Done,
    }

    struct Payout(u64, u64);

    struct Flip {
        payout: Payout,
        probability: u64,
    }
}
