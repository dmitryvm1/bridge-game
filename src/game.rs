use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}
impl Suit {
    pub const ALL: [Self; 4] = [Self::Clubs, Self::Diamonds, Self::Hearts, Self::Spades];
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Clubs => "♣",
            Self::Diamonds => "♦",
            Self::Hearts => "♥",
            Self::Spades => "♠",
        }
    }
    pub fn red(self) -> bool {
        matches!(self, Self::Diamonds | Self::Hearts)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Card {
    pub suit: Suit,
    pub rank: u8,
}
impl Card {
    pub fn label(self) -> String {
        format!(
            "{}{}",
            match self.rank {
                11 => "J".into(),
                12 => "Q".into(),
                13 => "K".into(),
                14 => "A".into(),
                n => n.to_string(),
            },
            self.suit.symbol()
        )
    }
    pub fn points(self) -> i32 {
        match self.rank {
            10 | 12 | 13 => 10,
            11 => 20,
            14 => 15,
            _ => 0,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Player {
    pub name: String,
    pub hand: Vec<Card>,
    pub score: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Playing,
    RoundOver(usize),
    GameOver(usize),
}
pub struct Game {
    pub players: Vec<Player>,
    pub deck: Vec<Card>,
    pub discard: Vec<Card>,
    pub turn: usize,
    pub dealer: usize,
    pub round: usize,
    pub suit: Suit,
    pub queen_rule: bool,
    pub phase: Phase,
    pub log: Vec<String>,
    pub last_deltas: Vec<i32>,
    rng: u64,
}
impl Game {
    pub fn new(count: usize, queen_rule: bool) -> Self {
        let mut game = Self {
            players: (0..count.clamp(2, 4))
                .map(|i| Player {
                    name: if i == 0 {
                        "You".into()
                    } else {
                        format!("Computer {i}")
                    },
                    hand: vec![],
                    score: 0,
                })
                .collect(),
            deck: vec![],
            discard: vec![],
            turn: 0,
            dealer: 0,
            round: 0,
            suit: Suit::Clubs,
            queen_rule,
            phase: Phase::Playing,
            log: vec![],
            last_deltas: vec![],
            rng: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64
                | 1,
        };
        game.deal();
        game
    }
    fn shuffle(&mut self) {
        for i in (1..self.deck.len()).rev() {
            self.rng ^= self.rng << 13;
            self.rng ^= self.rng >> 7;
            self.rng ^= self.rng << 17;
            self.deck.swap(i, self.rng as usize % (i + 1));
        }
    }
    pub fn note(&mut self, text: String) {
        self.log.push(text);
        if self.log.len() > 80 {
            self.log.remove(0);
        }
    }
    fn next(&self, p: usize) -> usize {
        (p + 1) % self.players.len()
    }
    pub fn top(&self) -> Card {
        *self.discard.last().expect("a round has a discard")
    }
    fn deal(&mut self) {
        self.round += 1;
        self.phase = Phase::Playing;
        self.last_deltas.clear();
        self.deck = Suit::ALL
            .into_iter()
            .flat_map(|suit| (6..=14).map(move |rank| Card { suit, rank }))
            .collect();
        self.discard.clear();
        self.shuffle();
        for p in &mut self.players {
            p.hand.clear();
        }
        for pass in 0..5 {
            for offset in 1..=self.players.len() {
                let p = (self.dealer + offset) % self.players.len();
                let c = self.deck.pop().unwrap();
                if pass == 4 && p == self.dealer {
                    self.discard.push(c);
                    self.suit = c.suit;
                } else {
                    self.players[p].hand.push(c);
                }
            }
        }
        self.turn = self.next(self.dealer);
        self.note(format!(
            "Round {} · {} deals {}",
            self.round,
            self.players[self.dealer].name,
            self.top().label()
        ));
        self.effects(self.dealer, &[self.top()]);
    }
    pub fn next_round(&mut self) {
        if matches!(self.phase, Phase::RoundOver(_)) {
            self.dealer = self.next(self.dealer);
            self.deal();
        }
    }
    pub fn legal(&self, card: Card) -> bool {
        if self.top().rank == 6 && card.rank == 6 {
            return false;
        }
        card.rank == 11
            || card.suit == self.suit
            || (self.top().rank != 11 && card.rank == self.top().rank)
    }
    pub fn can_play(&self) -> bool {
        self.players[self.turn].hand.iter().any(|&c| self.legal(c))
    }
    fn draw_one(&mut self, p: usize) -> bool {
        if self.deck.is_empty() && self.discard.len() > 1 {
            let top = self.discard.pop().unwrap();
            self.deck = std::mem::take(&mut self.discard);
            self.discard.push(top);
            self.shuffle();
            self.note("Discard pile reshuffled".into());
        }
        if let Some(c) = self.deck.pop() {
            self.players[p].hand.push(c);
            true
        } else {
            false
        }
    }
    fn penalty(&mut self, p: usize, count: usize) {
        let mut actual = 0;
        for _ in 0..count {
            if self.draw_one(p) {
                actual += 1;
            }
        }
        self.note(format!("{} draws {actual}", self.players[p].name));
    }
    fn effects(&mut self, actor: usize, cards: &[Card]) {
        self.turn = self.next(actor);
        match cards[0].rank {
            6 => self.turn = actor,
            7 => self.penalty(self.turn, cards.len()),
            8 | 14 => {
                let mut target = self.turn;
                for _ in cards {
                    if target == actor {
                        target = self.next(target);
                    }
                    if cards[0].rank == 8 {
                        self.penalty(target, 2);
                    }
                    self.note(format!("{} skips a turn", self.players[target].name));
                    target = self.next(target);
                }
                self.turn = target;
            }
            _ => {
                if self.queen_rule && cards.iter().any(|c| c.rank == 12 && c.suit == Suit::Spades) {
                    self.penalty(self.turn, 5);
                    self.turn = self.next(self.turn);
                }
            }
        }
    }
    pub fn play(&mut self, indices: &[usize], suit: Suit) -> Result<(), String> {
        if self.phase != Phase::Playing {
            return Err("The round has ended".into());
        }
        let hand = &self.players[self.turn].hand;
        if indices.is_empty() {
            return Err("Select a card first".into());
        }
        let mut sorted = indices.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        if sorted.len() != indices.len() || sorted.iter().any(|&i| i >= hand.len()) {
            return Err("Invalid card selection".into());
        }
        let cards: Vec<_> = indices.iter().map(|&i| hand[i]).collect();
        if !self.legal(cards[0]) || cards.iter().any(|c| c.rank != cards[0].rank) {
            return Err(
                "Lead with a matching suit or rank; additional cards must share its rank".into(),
            );
        }
        let actor = self.turn;
        for &i in sorted.iter().rev() {
            self.players[actor].hand.remove(i);
        }
        self.discard.extend(&cards);
        self.suit = if cards[0].rank == 11 {
            suit
        } else {
            cards.last().unwrap().suit
        };
        self.note(format!(
            "{} plays {}{}",
            self.players[actor].name,
            cards
                .iter()
                .map(|c| c.label())
                .collect::<Vec<_>>()
                .join(" "),
            if cards[0].rank == 11 {
                format!(" · {} chosen", suit.symbol())
            } else {
                String::new()
            }
        ));
        self.effects(actor, &cards);
        if self.players[actor].hand.is_empty() && cards[0].rank != 6 {
            self.finish(
                actor,
                if cards[0].rank == 11 {
                    -20 * cards.len() as i32
                } else {
                    0
                },
            );
        }
        Ok(())
    }
    pub fn draw(&mut self) -> Result<(), String> {
        if self.phase != Phase::Playing {
            return Err("The round has ended".into());
        }
        if self.can_play() {
            return Err("You have a legal move; play a card".into());
        }
        let p = self.turn;
        let cover = self.top().rank == 6;
        let mut count = 0;
        loop {
            if !self.draw_one(p) {
                break;
            }
            count += 1;
            if self.can_play() || !cover {
                break;
            }
        }
        self.note(format!(
            "{} draws {count}{}",
            self.players[p].name,
            if cover { " to cover the six" } else { "" }
        ));
        if !self.can_play() {
            self.turn = self.next(p);
        }
        // With no recyclable cards and no legal move anywhere, settle a blocked round.
        if self.deck.is_empty()
            && self.discard.len() == 1
            && !self
                .players
                .iter()
                .any(|p| p.hand.iter().any(|&c| self.legal(c)))
        {
            let winner = self
                .players
                .iter()
                .enumerate()
                .min_by_key(|(_, p)| p.hand.iter().map(|c| c.points()).sum::<i32>())
                .unwrap()
                .0;
            self.note("Table blocked: lowest hand penalty wins the round".into());
            self.finish(winner, 0);
        }
        Ok(())
    }
    fn finish(&mut self, winner: usize, bonus: i32) {
        self.last_deltas = self
            .players
            .iter()
            .enumerate()
            .map(|(i, p)| {
                if i == winner {
                    bonus
                } else {
                    p.hand.iter().map(|c| c.points()).sum()
                }
            })
            .collect();
        for (p, delta) in self.players.iter_mut().zip(&self.last_deltas) {
            p.score += delta;
            if p.score == 125 {
                p.score = 0;
            }
        }
        self.phase = self
            .players
            .iter()
            .position(|p| p.score > 125)
            .map(Phase::GameOver)
            .unwrap_or(Phase::RoundOver(winner));
        self.note(format!(
            "{} wins round {}",
            self.players[winner].name, self.round
        ));
    }
    pub fn bot_step(&mut self) {
        if self.phase != Phase::Playing || self.turn == 0 {
            return;
        }
        let hand = &self.players[self.turn].hand;
        if let Some(first) = hand
            .iter()
            .enumerate()
            .filter(|(_, c)| self.legal(**c))
            .max_by_key(|(_, c)| c.points())
            .map(|(i, _)| i)
        {
            let rank = hand[first].rank;
            let mut indices = vec![first];
            indices.extend((0..hand.len()).filter(|&i| i != first && hand[i].rank == rank));
            let suit = Suit::ALL
                .into_iter()
                .max_by_key(|s| hand.iter().filter(|c| c.suit == *s && c.rank != 11).count())
                .unwrap();
            self.play(&indices, suit).unwrap();
        } else {
            let _ = self.draw();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn c(rank: u8, suit: Suit) -> Card {
        Card { rank, suit }
    }
    fn table(n: usize, top: Card) -> Game {
        let mut g = Game::new(n, false);
        g.discard = vec![top];
        g.suit = top.suit;
        g.turn = 0;
        g.phase = Phase::Playing;
        for p in &mut g.players {
            p.hand = vec![c(9, Suit::Clubs), c(10, Suit::Hearts)];
            p.score = 0;
        }
        g
    }
    #[test]
    fn deal_conserves_deck() {
        for n in 2..=4 {
            let g = Game::new(n, false);
            assert_eq!(
                g.deck.len()
                    + g.discard.len()
                    + g.players.iter().map(|p| p.hand.len()).sum::<usize>(),
                36
            );
        }
    }
    #[test]
    fn six_cannot_cover_six() {
        let g = table(2, c(6, Suit::Hearts));
        assert!(!g.legal(c(6, Suit::Spades)));
        assert!(g.legal(c(11, Suit::Clubs)));
        assert!(g.legal(c(7, Suit::Hearts)));
    }
    #[test]
    fn computer_keeps_turn_until_its_six_is_covered() {
        let mut g = table(2, c(13, Suit::Hearts));
        g.turn = 1;
        g.players[1].hand = vec![c(6, Suit::Hearts), c(9, Suit::Clubs)];
        g.deck = vec![c(10, Suit::Hearts), c(7, Suit::Spades)];
        g.bot_step();
        assert_eq!(g.top(), c(6, Suit::Hearts));
        assert_eq!(g.turn, 1);
        g.bot_step();
        assert_eq!(g.turn, 1);
        assert_eq!(g.players[1].hand.len(), 3);
        g.bot_step();
        assert_eq!(g.top(), c(10, Suit::Hearts));
        assert_eq!(g.turn, 0);
    }
    #[test]
    fn final_sixes_require_a_cover_before_finishing() {
        let mut g = table(2, c(9, Suit::Hearts));
        g.players[0].hand = vec![c(6, Suit::Hearts), c(6, Suit::Clubs)];
        g.deck = vec![c(10, Suit::Clubs)];
        g.play(&[0, 1], Suit::Hearts).unwrap();
        assert_eq!(g.phase, Phase::Playing);
        assert_eq!(g.turn, 0);
        g.draw().unwrap();
        assert_eq!(g.turn, 0);
        g.play(&[0], Suit::Clubs).unwrap();
        assert_eq!(g.phase, Phase::RoundOver(0));
    }
    #[test]
    fn opening_six_must_be_covered_by_dealer() {
        let mut g = table(3, c(6, Suit::Hearts));
        g.effects(2, &[g.top()]);
        assert_eq!(g.turn, 2);
    }
    #[test]
    fn jack_requires_chosen_suit() {
        let mut g = table(2, c(11, Suit::Clubs));
        g.suit = Suit::Hearts;
        assert!(!g.legal(c(9, Suit::Clubs)));
        assert!(g.legal(c(9, Suit::Hearts)));
    }
    #[test]
    fn two_eights_skip_two_opponents() {
        let mut g = table(4, c(8, Suit::Clubs));
        g.players[0].hand = vec![c(8, Suit::Clubs), c(8, Suit::Hearts), c(9, Suit::Clubs)];
        g.play(&[0, 1], Suit::Clubs).unwrap();
        assert_eq!(g.turn, 3);
        assert_eq!(g.players[1].hand.len(), 4);
        assert_eq!(g.players[2].hand.len(), 4);
    }
    #[test]
    fn two_player_eights_return_to_actor() {
        let mut g = table(2, c(8, Suit::Clubs));
        g.players[0].hand = vec![c(8, Suit::Clubs), c(8, Suit::Hearts), c(9, Suit::Clubs)];
        g.play(&[0, 1], Suit::Clubs).unwrap();
        assert_eq!(g.turn, 0);
        assert_eq!(g.players[1].hand.len(), 6);
    }
    #[test]
    fn finishing_jacks_and_exact_reset() {
        let mut g = table(2, c(9, Suit::Clubs));
        g.players[0].hand = vec![c(11, Suit::Clubs), c(11, Suit::Hearts)];
        g.players[1].score = 115;
        g.play(&[0, 1], Suit::Spades).unwrap();
        assert_eq!(g.players[0].score, -40);
        assert_eq!(g.players[1].score, 0);
        assert_eq!(g.phase, Phase::RoundOver(0));
    }
    #[test]
    fn final_eight_applies_penalty_before_scoring() {
        let mut g = table(2, c(8, Suit::Clubs));
        g.players[0].hand = vec![c(8, Suit::Hearts)];
        g.deck = vec![c(14, Suit::Spades), c(13, Suit::Diamonds)];
        g.players[1].score = 100;
        g.play(&[0], Suit::Clubs).unwrap();
        assert_eq!(g.players[1].hand.len(), 4);
        assert!(g.deck.is_empty());
        assert_eq!(g.last_deltas, vec![0, 35]);
        assert_eq!(g.players[1].score, 135);
        assert_eq!(g.phase, Phase::GameOver(1));
    }
    #[test]
    fn final_eights_apply_all_penalties() {
        for n in 2..=4 {
            let mut g = table(n, c(8, Suit::Clubs));
            g.players[0].hand = vec![c(8, Suit::Hearts), c(8, Suit::Spades)];
            g.deck = vec![c(10, Suit::Clubs); 4];
            g.play(&[0, 1], Suit::Spades).unwrap();
            assert_eq!(g.phase, Phase::RoundOver(0));
            assert!(g.players[0].hand.is_empty());
            assert_eq!(g.players[1].hand.len(), if n == 2 { 6 } else { 4 });
            if n > 2 {
                assert_eq!(g.players[2].hand.len(), 4);
            }
            assert!(g.deck.is_empty());
        }
    }
    #[test]
    fn invalid_selection_is_atomic() {
        let mut g = table(2, c(9, Suit::Clubs));
        let before = g.players[0].hand.clone();
        assert!(g.play(&[0, 1], Suit::Hearts).is_err());
        assert_eq!(g.players[0].hand, before);
    }
    #[test]
    fn queen_penalty() {
        let mut g = table(3, c(9, Suit::Spades));
        g.queen_rule = true;
        g.players[0].hand = vec![c(12, Suit::Spades), c(9, Suit::Clubs)];
        g.play(&[0], Suit::Clubs).unwrap();
        assert_eq!(g.players[1].hand.len(), 7);
        assert_eq!(g.turn, 2);
    }
    #[test]
    fn six_draws_until_a_cover_and_retains_turn() {
        let mut g = table(2, c(6, Suit::Hearts));
        g.players[0].hand = vec![c(9, Suit::Clubs)];
        g.deck = vec![c(10, Suit::Hearts), c(7, Suit::Spades), c(6, Suit::Clubs)];
        g.draw().unwrap();
        assert_eq!(g.players[0].hand.len(), 4);
        assert_eq!(g.turn, 0);
        assert!(g.can_play());
    }
    #[test]
    fn seven_is_mandatory_but_does_not_skip() {
        let mut g = table(3, c(7, Suit::Clubs));
        g.players[0].hand = vec![c(7, Suit::Hearts), c(9, Suit::Clubs)];
        g.play(&[0], Suit::Clubs).unwrap();
        assert_eq!(g.players[1].hand.len(), 3);
        assert_eq!(g.turn, 1);
    }
    #[test]
    fn recycling_keeps_the_top_card() {
        let mut g = table(2, c(9, Suit::Hearts));
        g.players[0].hand = vec![c(10, Suit::Clubs)];
        g.deck.clear();
        g.discard = vec![c(7, Suit::Spades), c(9, Suit::Hearts)];
        g.draw().unwrap();
        assert_eq!(g.top(), c(9, Suit::Hearts));
        assert_eq!(g.players[0].hand.last(), Some(&c(7, Suit::Spades)));
        assert_eq!(g.turn, 1);
    }
    #[test]
    fn score_above_threshold_ends_game() {
        let mut g = table(2, c(9, Suit::Clubs));
        g.players[0].hand = vec![c(9, Suit::Hearts)];
        g.players[1].score = 120;
        g.play(&[0], Suit::Hearts).unwrap();
        assert_eq!(g.phase, Phase::GameOver(1));
        assert_eq!(g.players[1].score, 130);
    }
    #[test]
    fn simulations_terminate_and_conserve_cards() {
        for n in 2..=4 {
            for _ in 0..20 {
                let mut g = Game::new(n, true);
                for _ in 0..10000 {
                    if g.phase != Phase::Playing {
                        break;
                    }
                    assert_eq!(
                        g.deck.len()
                            + g.discard.len()
                            + g.players.iter().map(|p| p.hand.len()).sum::<usize>(),
                        36
                    );
                    let turn = g.turn;
                    if let Some(i) = g.players[turn].hand.iter().position(|&c| g.legal(c)) {
                        let s = g.players[turn].hand[i].suit;
                        g.play(&[i], s).unwrap();
                    } else {
                        g.draw().unwrap();
                    }
                }
                assert_ne!(g.phase, Phase::Playing);
            }
        }
    }
}
