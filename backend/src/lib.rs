pub mod api { }

pub mod game {
    pub struct Game { }

    impl Game { }

    enum Mode {
        Joining,
        Betting(Vec<Flip>),
    }

    /// self.0 to self.1 odds
    struct Payout(u64, u64);

    struct Flip {
        payout: Payout,
        /// percentage of success (out of 100)
        probability: u64,
    }

    struct EvaluatedFlip {
        payout: Payout,
        is_heads: bool,
    }

}
