pub mod api { }

pub mod game {
    use std::collections::HashMap;

    pub struct Game {
        players: HashMap<String, Player>,
        mode_iter: ModeIterator,
        mode: Mode,
    }

    impl Game { 

        pub fn new(flips: Vec<Vec<Flip>>) -> Self {
            Self {
                players: HashMap::new(),
                mode_iter: ModeIterator::new(flips),
                mode: Mode::Joining,
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
                                        let (num, denom) = results.iter().fold((1, 1), |(al, ar), elt| {
                                            (al * elt.payout.0, ar * elt.payout.1)
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

    #[derive(Clone)]
    struct Bet {
        amount: u64,
        flips: Vec<usize>,
    }

    #[derive(Clone)]
    enum Mode {
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
    #[derive(Clone, Copy)]
    struct Payout(u64, u64);

    #[derive(Clone, Copy)]
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

    #[derive(Clone, Copy)]
    struct EvaluatedFlip {
        payout: Payout,
        is_heads: bool,
    }

}
