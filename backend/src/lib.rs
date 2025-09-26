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
    }

    struct Player {
        stack: u64
    }

    impl Default for Player {
        fn default() -> Self {
            todo!()
        }
    }

    #[derive(PartialEq, Eq)]
    enum Mode {
        Joining,
        Betting,
        BetResult,
        Done,
    }
}
