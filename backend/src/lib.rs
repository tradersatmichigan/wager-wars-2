pub mod api {
    use std::{convert::Infallible, sync::Arc};

    use anyhow::Result;
    use axum::{extract::State, response::{sse::Event, IntoResponse, Sse}, Json};
    use axum_extra::extract::{cookie::Cookie, CookieJar};
    use serde::{Deserialize, Serialize};
    use tokio::sync::Mutex;
    use tokio_stream::{wrappers::BroadcastStream, Stream, StreamExt};

    use crate::game::{self, Bet, Game};

    #[derive(Clone)]
    pub struct AppState {
        game: Arc<Mutex<Game>>,
        key: String,
    }

    impl AppState {
        pub fn new(game: Game) -> Self {
            Self {
                game: Arc::new(Mutex::new(game)),
                key: rand::random::<u32>().to_string()
            }
        }
    }

    #[derive(Deserialize)]
    pub struct JoinRequest {
        name: String
    }

    pub async fn join_game(
        State(state): State<AppState>, 
        Json(request): Json<JoinRequest>, 
        jar: CookieJar
    ) -> impl IntoResponse
    {
        state.game.lock().await.add_player(request.name.clone())?;
        Ok(jar.add(Cookie::new(state.key, request.name)))
    }

    #[derive(Deserialize)]
    pub struct BetRequest {
        amount: u64,        
        indexes: Vec<usize>,
    }

    pub async fn place_bet(
        State(state): State<AppState>, 
        Json(request): Json<BetRequest>, 
        jar: CookieJar
        ) -> impl IntoResponse 
    {
        let player = match jar.get(&state.key) {
            Some(name) => name.value(),
            None => anyhow::bail!("You arent signed in")
        };

        state.game.lock().await.bet(player.to_string(), Bet::new(request.amount, request.indexes))?;

        Ok(())
    }

    #[derive(Serialize)]
    pub struct Update {
        mode: game::Mode,
        stack: Option<u64>,
    }

    pub async fn get_player_state(
        State(state): State<AppState>, 
        jar: CookieJar
        ) -> impl IntoResponse
    {
        let player = jar.get(&state.key).map(Cookie::value);

        let game = state.game.lock().await;

        Json(Update {
            mode: game.mode(),
            stack: player
                .and_then(|player| game.get_stack(player.to_string()).ok())
        })
    }

    pub async fn get_events(State(state): State<AppState>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
        let rx = state.game.lock().await.subscribe();

        let stream = BroadcastStream::new(rx).map(|_| {
            Ok(Event::default())
        }); 

        Sse::new(stream).keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(std::time::Duration::from_secs(30))
                .text("ping")
        )
    }
}

pub mod game {
    use std::collections::HashMap;

    use anyhow::Result;
    use serde::Serialize;
    use tokio::sync::broadcast;

    pub struct Game {
        players: HashMap<String, Player>,
        mode_iter: ModeIterator,
        mode: Mode,
        tx: broadcast::Sender<()>,
    }

    impl Game { 

        pub fn new(flips: Vec<Vec<Flip>>) -> Self {
            Self {
                players: HashMap::new(),
                mode_iter: ModeIterator::new(flips),
                mode: Mode::Joining,
                tx: broadcast::Sender::new(1000),
            }
        }

        pub fn subscribe(&self) -> broadcast::Receiver<()> {
            self.tx.subscribe()
        }

        pub fn mode(&self) -> Mode {
            self.mode.clone()
        }

        pub fn add_player(&mut self, name: String) -> Result<()> {
            if !matches!(self.mode, Mode::Joining) {
                anyhow::bail!("This game is already in progress");
            }

            if self.players.contains_key(&name) {
                anyhow::bail!("This name is already taken");
            }

            self.players.insert(name, Player::default());
            Ok(())
        }

        pub fn get_stack(&self, player: String) -> Result<u64> {
            let player = self.players.get(&player).ok_or(anyhow::anyhow!("Not signed in"))?;
            Ok(player.stack)
        }

        pub fn bet(&mut self, player: String, bet: Bet) -> Result<()> {
            let flips = match self.mode.clone() {
                Mode::Betting(flips) => flips,
                _ => anyhow::bail!("You cannot bet right now")
            };

            if bet.flips.iter().any(|idx| *idx > flips.len()) {
                anyhow::bail!("Invalid bet slip")
            }

            let player = self.players.get_mut(&player);

            match player {
                Some(player) => {
                    if player.stack < bet.amount {
                        anyhow::bail!("You cannot bet more than your stack");
                    }

                    player.current_bet = Some(bet);
                    Ok(())
                }
                None => anyhow::bail!("Invalid player")
            }
        }

        pub fn tick(&mut self) {
            match self.mode_iter.next() {
                Some(next) => {
                    self.mode = next;

                    match self.mode.clone() {
                        Mode::Betting(_) => {
                            for (_, player) in self.players.iter_mut() {
                                player.current_bet = None;
                            }
                        } // If betting

                        Mode::BetResults(results) => {
                            for (_, player) in self.players.iter_mut() {
                                if let Some(bet) = player.current_bet.clone() {
                                    if bet.flips.iter().all(|idx| { results[*idx].is_heads }) {
                                        let (num, denom) = bet.flips.iter().fold((1, 1), |(al, ar), bet_idx| {
                                            let Payout(l, r) = results[*bet_idx].payout;
                                            (al * l, ar * r)
                                        });
                                        player.stack += bet.amount * num / denom;
                                    } // If hit
                                    else {
                                        player.stack -= bet.amount;
                                    } // If missed parlay
                                } // if placed a bet
                            } // for player
                        } // if results

                        _ => {} // noop for safety
                    }

                }

                None => {
                    self.mode = Mode::Results;
                }
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
                current_bet: None
            }
        }
    }

    #[derive(Clone)]
    pub struct Bet {
        amount: u64,
        flips: Vec<usize>,
    }

    impl Bet {
        pub fn new(amount: u64, flips: Vec<usize>) -> Self {
            Self {
                amount,
                flips
            }
        }
    }

    #[derive(Clone, Serialize)]
    pub enum Mode {
        Joining,
        Betting(Vec<Flip>),
        BetResults(Vec<EvaluatedFlip>),
        Results,
    }

    struct ModeIterator {
        flips: Vec<Vec<Flip>>,
        idx: usize,
        first: bool,
    }

    impl ModeIterator {
        fn new(flips: Vec<Vec<Flip>>) -> Self {
            Self {
                flips,
                idx: 0,
                first: true,
            }
        }
    }

    impl Iterator for ModeIterator {
        type Item = Mode;

        fn next(&mut self) -> Option<Self::Item> {
            if self.idx >= self.flips.len() {
                None
            } else if self.first {
                self.first = false; 
                Some(Mode::Betting(self.flips[self.idx].clone()))
            } else {
                let res: Vec<EvaluatedFlip> 
                    = self.flips[self.idx]
                    .clone()
                    .into_iter()
                    .map(|f| f.into())
                    .collect() ;

                self.first = true;
                self.idx += 1;
                Some(Mode::BetResults(res))
            }
        }
    }

    /// self.0 to self.1 odds
    #[derive(Clone, Copy, Serialize)]
    struct Payout(u64, u64);

    #[derive(Clone, Copy, Serialize)]
    pub struct Flip {
        payout: Payout,
        /// percentage of success (out of 100)
        probability: u64,
    }

    impl Into<EvaluatedFlip> for Flip {
        fn into(self) -> EvaluatedFlip {
            let is_heads = (rand::random::<u64>() % 100) < self.probability;
            EvaluatedFlip {
                payout: self.payout,
                is_heads
            }
        }
    }

    #[derive(Clone, Copy, Serialize)]
    pub struct EvaluatedFlip {
        payout: Payout,
        is_heads: bool,
    }

}
