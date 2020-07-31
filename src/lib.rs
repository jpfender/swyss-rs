#![crate_name = "swyss"]
use core::cell::RefCell;
use rand::prelude::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use range_check::{Check, OutOfRangeError};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;

/// Represents a player and their match history
pub struct Player {
    pub uuid: uuid::Uuid,
    pub name: String,
    pub match_points: u32,
    pub game_points: u32,
    pub matches_played: u32,
    pub games_played: u32,
    pub opponents: Vec<Rc<RefCell<Player>>>,
    pub has_bye: bool,
}

impl Player {
    /// Returns a new player with the given name
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name of the player
    pub fn new(name: &str) -> Player {
        Player {
            uuid: Uuid::new_v4(),
            name: String::from(name),
            match_points: 0,
            game_points: 0,
            matches_played: 0,
            games_played: 0,
            opponents: Vec::new(),
            has_bye: false,
        }
    }

    /// Makes the player lose a single game. Has no effect other than increasing the number of
    /// games played, as lost games are not tracked explicitly.
    ///
    /// # Example
    ///
    /// ```
    /// use swyss::Player;
    /// let mut player = Player::new("Loser");
    /// player.lose_game();
    /// assert_eq!(player.games_played, 1);
    /// assert_eq!(player.game_points, 0);
    /// ```
    pub fn lose_game(&mut self) {
        self.games_played += 1;
    }

    /// Makes the player draw a single game. Increases the number of games played and adds one game
    /// point.
    ///
    /// # Example
    ///
    /// ```
    /// use swyss::Player;
    /// let mut player = Player::new("Drawer");
    /// player.draw_game();
    /// assert_eq!(player.games_played, 1);
    /// assert_eq!(player.game_points, 1);
    /// ```
    pub fn draw_game(&mut self) {
        self.games_played += 1;
        self.game_points += 1;
    }

    /// Makes the player win a single game. Increases the number of games played and adds three
    /// game points.
    ///
    /// # Example
    ///
    /// ```
    /// use swyss::Player;
    /// let mut player = Player::new("Winner");
    /// player.win_game();
    /// assert_eq!(player.games_played, 1);
    /// assert_eq!(player.game_points, 3);
    /// ```
    pub fn win_game(&mut self) {
        self.games_played += 1;
        self.game_points += 3;
    }

    /// Makes the player lose a single match. Has no effect other than increasing the number of
    /// matches played, as lost matches are not tracked explicitly.
    ///
    /// # Example
    ///
    /// ```
    /// use swyss::Player;
    /// let mut player = Player::new("Loser");
    /// player.lose_match();
    /// assert_eq!(player.matches_played, 1);
    /// assert_eq!(player.match_points, 0);
    /// ```
    pub fn lose_match(&mut self) {
        self.matches_played += 1;
    }

    /// Makes the player draw a single match. Increases the number of matches played and adds one
    /// match point.
    ///
    /// # Example
    ///
    /// ```
    /// use swyss::Player;
    /// let mut player = Player::new("Drawer");
    /// player.draw_match();
    /// assert_eq!(player.matches_played, 1);
    /// assert_eq!(player.match_points, 1);
    /// ```
    pub fn draw_match(&mut self) {
        self.matches_played += 1;
        self.match_points += 1;
    }

    /// Makes the player win a single match. Increases the number of matches played and adds three
    /// match points.
    ///
    /// # Example
    ///
    /// ```
    /// use swyss::Player;
    /// let mut player = Player::new("Winner");
    /// player.win_match();
    /// assert_eq!(player.matches_played, 1);
    /// assert_eq!(player.match_points, 3);
    /// ```
    pub fn win_match(&mut self) {
        self.matches_played += 1;
        self.match_points += 3;
    }

    /// Awards the player a bye. The player is considered to have won their match 2-0. No opponent
    /// is added to the `opponents` vector. The player is recorded as having received a bye so that
    /// the tournament manager can check that no player is awarded more than one bye.
    ///
    /// # Example
    ///
    /// ```
    /// use swyss::Player;
    /// let mut player = Player::new("Byer");
    /// player.bye();
    /// assert_eq!(player.games_played, 2);
    /// assert_eq!(player.game_points, 6);
    /// assert_eq!(player.matches_played, 1);
    /// assert_eq!(player.match_points, 3);
    /// assert_eq!(player.opponents.len(), 0);
    /// assert!(player.has_bye);
    /// ```
    pub fn bye(&mut self) {
        self.win_game();
        self.win_game();
        self.win_match();
        self.has_bye = true;
    }

    /// Calculates the player's match win percentage, i.e. accumulated match points divided by
    /// total match points possible in those rounds. Minimum MWP returned is always 1/3 to reduce
    /// the impact of low performance on `opponents_match_win_percentage()`.
    pub fn match_win_percentage(&self) -> f64 {
        let default = 1.0 / 3.0;
        if self.matches_played == 0 {
            default
        } else {
            default.max(self.match_points as f64 / (3.0 * self.matches_played as f64))
        }
    }

    /// Calculates the player's game win percentage, i.e. accumulated game points divided by total
    /// game points possible in those rounds. Minimum GWP returned is always 1/3 to reduce the
    /// impact of low performance on `opponents_game_win_percentage()`.
    pub fn game_win_percentage(&self) -> f64 {
        let default = 1.0 / 3.0;
        if self.games_played == 0 {
            default
        } else {
            default.max(self.game_points as f64 / (3.0 * self.games_played as f64))
        }
    }

    /// Calculates the player's opponents' match win percentage, i.e. the average match win
    /// percentage of all opponents the player faced, ignoring byes.
    pub fn opponents_match_win_percentage(&self) -> f64 {
        let mut omwp = 0.0;

        for opp in &self.opponents {
            omwp += opp.borrow().match_win_percentage();
        }

        omwp / self.opponents.len() as f64
    }

    /// Calculates the player's opponents' game win percentage, i.e. the average game win
    /// percentage of all opponents the player faced, ignoring byes.
    pub fn opponents_game_win_percentage(&self) -> f64 {
        let mut omwp = 0.0;

        for opp in &self.opponents {
            omwp += opp.borrow().game_win_percentage();
        }

        omwp / self.opponents.len() as f64
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Player) -> bool {
        self.uuid == other.uuid
    }
}

pub enum PlayerSide {
    Home,
    Away,
}

pub struct Pairing {
    uuid: uuid::Uuid,
    home: Rc<RefCell<Player>>,
    away: Rc<RefCell<Player>>,
}

impl Pairing {
    /// Creates a new pairing between two players
    ///
    /// # Arguments
    ///
    /// * `home` - The left-hand player, wrapped in a `RefCell`
    /// * `away` - The right-hand player, wrapped in a `RefCell`
    pub fn new(home: Rc<RefCell<Player>>, away: Rc<RefCell<Player>>) -> Pairing {
        let uuid = Uuid::new_v4();
        let home = Rc::clone(&home);
        let away = Rc::clone(&away);
        home.borrow_mut().opponents.push(Rc::clone(&away));
        away.borrow_mut().opponents.push(Rc::clone(&home));
        Pairing { uuid, home, away }
    }

    /// Registers a won game for one of the players. Implies registering a lost game for the other
    /// player.
    ///
    /// # Arguments
    ///
    /// `side` - A `PlayerSide` that specifies whether the home or away player won the game
    pub fn win_game(&self, side: PlayerSide) {
        let (winner, loser) = match side {
            PlayerSide::Home => (&self.home, &self.away),
            PlayerSide::Away => (&self.away, &self.home),
        };

        winner.borrow_mut().win_game();
        loser.borrow_mut().lose_game();
    }

    /// Registers a drawn game.
    pub fn draw_game(&self) {
        &self.home.borrow_mut().draw_game();
        &self.away.borrow_mut().draw_game();
    }

    pub fn end_match(
        &self,
        home_score: u8,
        away_score: u8,
        drawn: u8,
    ) -> Result<(), OutOfRangeError<u8>> {
        // Ensure that game scores are valid both individually and overall
        home_score.check_range(0..3)?;
        away_score.check_range(0..3)?;
        drawn.check_range(0..4)?;

        // At least one game needs to have been completed, even if it's a draw
        (home_score + away_score + drawn).check_range(1..4)?;

        for _ in 0..home_score {
            self.win_game(PlayerSide::Home);
        }

        for _ in 0..away_score {
            self.win_game(PlayerSide::Away);
        }

        for _ in 0..drawn {
            self.draw_game();
        }

        if home_score > away_score {
            &self.home.borrow_mut().win_match();
            &self.away.borrow_mut().lose_match();
        } else if away_score > home_score {
            &self.home.borrow_mut().lose_match();
            &self.away.borrow_mut().win_match();
        } else {
            &self.home.borrow_mut().draw_match();
            &self.away.borrow_mut().draw_match();
        }

        Ok(())
    }
}

/// Manages the whole tournament. Holds players and their ranking and constructs pairings on demand
pub struct Tournament {
    pub rounds: u32,
    pub current_round: u32,
    pub players: Vec<Rc<RefCell<Player>>>,
    pub pairings: HashMap<uuid::Uuid, Pairing>,
    pub needs_bye: bool,
    rng: ThreadRng,
}

/// Recording the result of a pairing can fail for one of two reasons: Either the pairing does not
/// exist, or the supplied results are invalid
pub enum PairingResultError {
    NotFound(uuid::Uuid),
    OutOfRange(u8),
}

impl Tournament {
    pub fn new(players: Vec<Rc<RefCell<Player>>>) -> Tournament {
        let num_players = players.len();
        let rounds = (num_players as f64).log2().ceil() as u32;
        let needs_bye = if num_players % 2 == 0 { false } else { true };

        Tournament {
            players,
            rounds,
            current_round: 0,
            pairings: HashMap::with_capacity(num_players / 2),
            needs_bye,
            rng: thread_rng(),
        }
    }

    /// Grants a player a bye if the number of player is odd, otherwise returns `None`. Ensures
    /// that a player is granted at most one bye during a tournament. Removes the player who got
    /// the bye from the player list and returns them so they can be re-entered into the player
    /// list after pairings are complete.
    fn grant_bye(&mut self) -> Option<Rc<RefCell<Player>>> {
        if self.needs_bye {
            self.players.shuffle(&mut self.rng);

            // Get all players who have not yet received a bye
            let iter = self.players.iter().cloned().filter(|x| !x.borrow().has_bye);

            // Get the player with the lowest match points among those players
            let bye = iter.min_by_key(|x| x.borrow().match_points);

            if let Some(bye) = bye {
                let mut i = 0;
                while i < self.players.len() {
                    if self.players[i] == bye {
                        self.players[i].borrow_mut().bye();
                        return Some(self.players.remove(i));
                    }
                    i += 1;
                }
            }
        }

        None
    }

    /// Advances the tournament by one round. If there are still rounds left to play, construct new
    /// player pairings based on match points and return them. If there is an uneven number of
    /// player, the lowest-ranked player who has not yet received a bye receives a bye.
    pub fn next_round(&mut self) -> Option<Vec<(uuid::Uuid, String, String)>> {
        self.current_round += 1;
        if self.current_round > self.rounds {
            return None;
        }

        let bye = self.grant_bye();

        let mut player_queue;

        let mut ret: Vec<(uuid::Uuid, String, String)> =
            Vec::with_capacity(self.pairings.capacity());

        let mut repeat = true;

        while repeat {
            player_queue = self.players.to_vec();

            player_queue.shuffle(&mut self.rng);
            player_queue.sort_by(|a, b| a.borrow().match_points.cmp(&b.borrow().match_points));

            self.pairings.clear();
            ret.clear();

            while let Some(home) = player_queue.pop() {
                if player_queue.len() == 0 {
                    break;
                }

                let mut away;
                let mut i = player_queue.len() - 1;
                loop {
                    away = Rc::clone(&player_queue[i]);
                    if !home.borrow().opponents.contains(&away) {
                        player_queue.remove(i);
                        break;
                    }

                    if i == 0 {
                        break;
                    }

                    i -= 1;
                }

                let home = Rc::clone(&home);
                let away = Rc::clone(&away);
                let pair = Pairing::new(home, away);

                let uuid = pair.uuid;
                let home_str = String::from(&pair.home.borrow().name);
                let away_str = String::from(&pair.away.borrow().name);

                self.pairings.insert(uuid, pair);
                ret.push((uuid, home_str, away_str));
            }

            if self.pairings.len() == self.players.len() / 2 {
                repeat = false;
            }
        }

        ret.shuffle(&mut self.rng);

        if let Some(bye) = bye {
            self.players.push(bye);
        }

        Some(ret)
    }

    /// Record the result of a pairing, specified by its UUID. Basically just a wrapper around
    /// `Pairing::end_match()`, extended by the `NotFound` error type.
    pub fn end_match(
        &self,
        uuid: uuid::Uuid,
        home_score: u8,
        away_score: u8,
        drawn: u8,
    ) -> Result<(), PairingResultError> {
        if let Some(pair) = self.pairings.get(&uuid) {
            return match pair.end_match(home_score, away_score, drawn) {
                Ok(_) => Ok(()),
                Err(e) => Err(PairingResultError::OutOfRange(e.outside_value)),
            };
        }

        Err(PairingResultError::NotFound(uuid))
    }

    /// Rank all players using all tiebreakers. This only needs to be called if the ranking
    /// actually needs to be displayed (i.e. at the end of the tournament) or if results between
    /// rounds are desired; it is not necessary when progressing rounds as `next_round()`
    /// automatically performs a simpler ranking using just match points before creating new
    /// pairings.
    pub fn ranking(&mut self) -> Vec<Rc<RefCell<Player>>> {
        // Start with a shuffle so that any previous order does not affect the new order in case of
        // full ties
        self.players.shuffle(&mut self.rng);

        // Vec::sort_by() is stable, so we start with the last tiebreaker and sort upwards from
        // there.

        self.players.sort_by(|a, b| {
            b.borrow()
                .opponents_game_win_percentage()
                .partial_cmp(&a.borrow().opponents_game_win_percentage())
                .unwrap_or(Ordering::Equal)
        });
        self.players.sort_by(|a, b| {
            b.borrow()
                .game_win_percentage()
                .partial_cmp(&a.borrow().game_win_percentage())
                .unwrap_or(Ordering::Equal)
        });
        self.players.sort_by(|a, b| {
            b.borrow()
                .opponents_match_win_percentage()
                .partial_cmp(&a.borrow().opponents_match_win_percentage())
                .unwrap_or(Ordering::Equal)
        });
        self.players.sort_by(|a, b| {
            b.borrow()
                .match_points
                .partial_cmp(&a.borrow().match_points)
                .unwrap_or(Ordering::Equal)
        });

        self.players.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    /// Player goes 5-2-1
    fn mwp_521() {
        let mut player = Player::new("5-2-1");

        for _ in 0..5 {
            player.win_match();
        }

        player.lose_match();
        player.lose_match();

        player.draw_match();

        assert_eq!(player.match_points, 16);
        assert_eq!(player.matches_played, 8);
        assert_eq!(player.match_win_percentage(), 2.0 / 3.0);
    }

    #[test]
    /// Player goes 1-3-0, then withdraws from the tournament
    fn mwp_130_drop() {
        let mut player = Player::new("1-3-0-Drop");

        player.win_match();

        for _ in 0..3 {
            player.lose_match();
        }

        assert_eq!(player.match_points, 3);
        assert_eq!(player.matches_played, 4);
        assert_eq!(player.match_win_percentage(), 1.0 / 3.0);
    }

    #[test]
    /// Player gets a bye, goes 3-2-0 overall, then withdraws
    fn mwp_bye_320_drop() {
        let mut player = Player::new("Bye-3-2-0-Drop");

        player.bye();

        for _ in 0..2 {
            player.win_match();
            player.lose_match();
        }

        assert_eq!(player.match_points, 9);
        assert_eq!(player.matches_played, 5);
        assert_eq!(player.match_win_percentage(), 0.6);
    }

    #[test]
    /// Player goes 2-0, 2-1, 1-2, 2-0
    fn gwp_21_10() {
        let player = Rc::new(RefCell::new(Player::new("21-10")));

        let o1 = Rc::new(RefCell::new(Player::new("Opponent 1")));
        let o2 = Rc::new(RefCell::new(Player::new("Opponent 2")));
        let o3 = Rc::new(RefCell::new(Player::new("Opponent 3")));
        let o4 = Rc::new(RefCell::new(Player::new("Opponent 4")));

        // 2-0 (6 points)
        let pair = Pairing::new(Rc::clone(&player), o1);
        assert!(pair.end_match(2, 0, 0).is_ok());

        // 2-1 (6 points)
        let pair = Pairing::new(Rc::clone(&player), o2);
        assert!(pair.end_match(2, 1, 0).is_ok());

        // 1-2 (3 points)
        let pair = Pairing::new(Rc::clone(&player), o3);
        assert!(pair.end_match(1, 2, 0).is_ok());

        // 2-0 (6 points)
        let pair = Pairing::new(Rc::clone(&player), o4);
        assert!(pair.end_match(2, 0, 0).is_ok());

        let player = player.borrow();

        assert_eq!(player.opponents.len(), 4);
        assert_eq!(player.game_points, 21);
        assert_eq!(player.games_played, 10);
        assert_eq!(player.game_win_percentage(), 0.7);
    }

    #[test]
    /// Player goes 1-2, 1-2, 0-2, 1-2
    fn gwp_9_11() {
        let player = Rc::new(RefCell::new(Player::new("9-11")));

        let o1 = Rc::new(RefCell::new(Player::new("Opponent 1")));
        let o2 = Rc::new(RefCell::new(Player::new("Opponent 2")));
        let o3 = Rc::new(RefCell::new(Player::new("Opponent 3")));
        let o4 = Rc::new(RefCell::new(Player::new("Opponent 4")));

        // 1-2 (3 points)
        let pair = Pairing::new(Rc::clone(&player), o1);
        assert!(pair.end_match(1, 2, 0).is_ok());

        // 1-2 (3 points)
        let pair = Pairing::new(Rc::clone(&player), o2);
        assert!(pair.end_match(1, 2, 0).is_ok());

        // 0-2 (0 points)
        let pair = Pairing::new(Rc::clone(&player), o3);
        assert!(pair.end_match(0, 2, 0).is_ok());

        // 1-2 (3 points)
        let pair = Pairing::new(Rc::clone(&player), o4);
        assert!(pair.end_match(1, 2, 0).is_ok());

        let player = player.borrow();

        assert_eq!(player.opponents.len(), 4);
        assert_eq!(player.game_points, 9);
        assert_eq!(player.games_played, 11);
        assert_eq!(player.game_win_percentage(), 1.0 / 3.0);
    }

    #[test]
    /// Player goes 6-2-0, their opponents having gone 4-4-0, 7-1-0, 1-3-1, 3-3-1, 6-2-0, 5-2-1,
    /// 4-3-1, and 6-1-1
    fn omwp_normal() {
        let mut player = Player::new("Normal");

        // Opponent 1 goes 4-4-0
        let mut o1 = Player::new("Opponent 1");
        for _ in 0..4 {
            o1.win_match();
            o1.lose_match();
        }
        assert_eq!(o1.match_win_percentage(), 0.5);
        player.opponents.push(Rc::new(RefCell::new(o1)));

        // Opponent 2 goes 7-1-0
        let mut o2 = Player::new("Opponent 2");
        for _ in 0..7 {
            o2.win_match();
        }
        o2.lose_match();
        assert_eq!(o2.match_win_percentage(), 21.0 / 24.0);
        player.opponents.push(Rc::new(RefCell::new(o2)));

        // Opponent 3 goes 1-3-1
        let mut o3 = Player::new("Opponent 3");
        o3.win_match();
        for _ in 0..3 {
            o3.lose_match();
        }
        o3.draw_match();
        assert_eq!(o3.match_win_percentage(), 1.0 / 3.0);
        player.opponents.push(Rc::new(RefCell::new(o3)));

        // Opponent 4 goes 3-3-1
        let mut o4 = Player::new("Opponent 4");
        for _ in 0..3 {
            o4.win_match();
            o4.lose_match();
        }
        o4.draw_match();
        assert_eq!(o4.match_win_percentage(), 10.0 / 21.0);
        player.opponents.push(Rc::new(RefCell::new(o4)));

        // Opponent 5 goes 6-2-0
        let mut o5 = Player::new("Opponent 5");
        for _ in 0..6 {
            o5.win_match();
        }
        o5.lose_match();
        o5.lose_match();
        assert_eq!(o5.match_win_percentage(), 18.0 / 24.0);
        player.opponents.push(Rc::new(RefCell::new(o5)));

        // Opponent 6 goes 5-2-1
        let mut o6 = Player::new("Opponent 6");
        for _ in 0..5 {
            o6.win_match();
        }
        o6.lose_match();
        o6.lose_match();
        o6.draw_match();
        assert_eq!(o6.match_win_percentage(), 16.0 / 24.0);
        player.opponents.push(Rc::new(RefCell::new(o6)));

        // Opponent 7 goes 4-3-1
        let mut o7 = Player::new("Opponent 7");
        for _ in 0..4 {
            o7.win_match();
        }
        for _ in 0..3 {
            o7.lose_match();
        }
        o7.draw_match();
        assert_eq!(o7.match_win_percentage(), 13.0 / 24.0);
        player.opponents.push(Rc::new(RefCell::new(o7)));

        // Opponent 8 goes 6-1-1
        let mut o8 = Player::new("Opponent 8");
        for _ in 0..6 {
            o8.win_match();
        }
        o8.lose_match();
        o8.draw_match();
        assert_eq!(o8.match_win_percentage(), 19.0 / 24.0);
        player.opponents.push(Rc::new(RefCell::new(o8)));

        let expected_omwp = (12.0 / 24.0
            + 21.0 / 24.0
            + 1.0 / 3.0
            + 10.0 / 21.0
            + 18.0 / 24.0
            + 16.0 / 24.0
            + 13.0 / 24.0
            + 19.0 / 24.0)
            / 8.0;

        assert_eq!(player.opponents_match_win_percentage(), expected_omwp);
    }

    #[test]
    /// Player goes 6-2-0, their opponents having been bye, 7-1-0, 1-3-1, 3-3-1, 6-2-0, 5-2-1,
    /// 4-3-1, and 6-1-1
    fn omwp_with_bye() {
        let mut player = Player::new("Bye");

        // Player gets bye in round 1
        player.bye();

        // Opponent 2 goes 7-1-0
        let mut o2 = Player::new("Opponent 2");
        for _ in 0..7 {
            o2.win_match();
        }
        o2.lose_match();
        assert_eq!(o2.match_win_percentage(), 21.0 / 24.0);
        player.opponents.push(Rc::new(RefCell::new(o2)));

        // Opponent 3 goes 1-3-1
        let mut o3 = Player::new("Opponent 3");
        o3.win_match();
        for _ in 0..3 {
            o3.lose_match();
        }
        o3.draw_match();
        assert_eq!(o3.match_win_percentage(), 1.0 / 3.0);
        player.opponents.push(Rc::new(RefCell::new(o3)));

        // Opponent 4 goes 3-3-1
        let mut o4 = Player::new("Opponent 4");
        for _ in 0..3 {
            o4.win_match();
            o4.lose_match();
        }
        o4.draw_match();
        assert_eq!(o4.match_win_percentage(), 10.0 / 21.0);
        player.opponents.push(Rc::new(RefCell::new(o4)));

        // Opponent 5 goes 6-2-0
        let mut o5 = Player::new("Opponent 5");
        for _ in 0..6 {
            o5.win_match();
        }
        o5.lose_match();
        o5.lose_match();
        assert_eq!(o5.match_win_percentage(), 18.0 / 24.0);
        player.opponents.push(Rc::new(RefCell::new(o5)));

        // Opponent 6 goes 5-2-1
        let mut o6 = Player::new("Opponent 6");
        for _ in 0..5 {
            o6.win_match();
        }
        o6.lose_match();
        o6.lose_match();
        o6.draw_match();
        assert_eq!(o6.match_win_percentage(), 16.0 / 24.0);
        player.opponents.push(Rc::new(RefCell::new(o6)));

        // Opponent 7 goes 4-3-1
        let mut o7 = Player::new("Opponent 7");
        for _ in 0..4 {
            o7.win_match();
        }
        for _ in 0..3 {
            o7.lose_match();
        }
        o7.draw_match();
        assert_eq!(o7.match_win_percentage(), 13.0 / 24.0);
        player.opponents.push(Rc::new(RefCell::new(o7)));

        // Opponent 8 goes 6-1-1
        let mut o8 = Player::new("Opponent 8");
        for _ in 0..6 {
            o8.win_match();
        }
        o8.lose_match();
        o8.draw_match();
        assert_eq!(o8.match_win_percentage(), 19.0 / 24.0);
        player.opponents.push(Rc::new(RefCell::new(o8)));

        let expected_omwp = (21.0 / 24.0
            + 1.0 / 3.0
            + 10.0 / 21.0
            + 18.0 / 24.0
            + 16.0 / 24.0
            + 13.0 / 24.0
            + 19.0 / 24.0)
            / 7.0;

        assert_eq!(player.opponents_match_win_percentage(), expected_omwp);
    }

    #[test]
    /// Alice 2-0 Bob
    fn pairing_games_20() {
        let alice = Rc::new(RefCell::new(Player::new("Alice")));
        let bob = Rc::new(RefCell::new(Player::new("Bob")));
        let alice_clone = Rc::clone(&alice);
        let bob_clone = Rc::clone(&bob);

        let pair = Pairing::new(alice, bob);

        pair.win_game(PlayerSide::Home);
        pair.win_game(PlayerSide::Home);

        let alice = alice_clone.borrow();
        let bob = bob_clone.borrow();

        assert_eq!(alice.game_points, 6);
        assert_eq!(bob.game_points, 0);

        assert_eq!(alice.games_played, 2);
        assert_eq!(bob.games_played, 2);

        // Even though a 2-0 is technically a won match, it is the tournament manager's
        // responsibility to mark the match as completed and won, so we expect no change here
        assert_eq!(alice.matches_played, 0);
        assert_eq!(bob.matches_played, 0);
        assert_eq!(alice.match_points, 0);
        assert_eq!(bob.match_points, 0);

        assert_eq!(alice.opponents.len(), 1);
        assert_eq!(bob.opponents.len(), 1);
    }

    #[test]
    /// Charlie 2-1 Dan
    fn pairing_games_21() {
        let charlie = Rc::new(RefCell::new(Player::new("Charlie")));
        let dan = Rc::new(RefCell::new(Player::new("Dan")));
        let charlie_clone = Rc::clone(&charlie);
        let dan_clone = Rc::clone(&dan);

        let pair = Pairing::new(charlie, dan);

        pair.win_game(PlayerSide::Home);
        pair.win_game(PlayerSide::Away);
        pair.win_game(PlayerSide::Home);

        let charlie = charlie_clone.borrow();
        let dan = dan_clone.borrow();

        assert_eq!(charlie.game_points, 6);
        assert_eq!(dan.game_points, 3);

        assert_eq!(charlie.games_played, 3);
        assert_eq!(dan.games_played, 3);

        assert_eq!(charlie.matches_played, 0);
        assert_eq!(dan.matches_played, 0);
        assert_eq!(charlie.match_points, 0);
        assert_eq!(dan.match_points, 0);

        assert_eq!(charlie.opponents.len(), 1);
        assert_eq!(dan.opponents.len(), 1);
    }

    #[test]
    /// Eve 2-0-1 Frank
    fn pairing_games_201() {
        let eve = Rc::new(RefCell::new(Player::new("Eve")));
        let frank = Rc::new(RefCell::new(Player::new("Frank")));
        let eve_clone = Rc::clone(&eve);
        let frank_clone = Rc::clone(&frank);

        let pair = Pairing::new(eve, frank);

        pair.win_game(PlayerSide::Home);
        pair.draw_game();
        pair.win_game(PlayerSide::Home);

        let eve = eve_clone.borrow();
        let frank = frank_clone.borrow();

        assert_eq!(eve.game_points, 7);
        assert_eq!(frank.game_points, 1);

        assert_eq!(eve.games_played, 3);
        assert_eq!(frank.games_played, 3);

        assert_eq!(eve.matches_played, 0);
        assert_eq!(frank.matches_played, 0);
        assert_eq!(eve.match_points, 0);
        assert_eq!(frank.match_points, 0);

        assert_eq!(eve.opponents.len(), 1);
        assert_eq!(frank.opponents.len(), 1);
    }

    #[test]
    /// Alice 2-0 Bob
    fn pairing_match_20() {
        let alice = Rc::new(RefCell::new(Player::new("Alice")));
        let bob = Rc::new(RefCell::new(Player::new("Bob")));
        let alice_clone = Rc::clone(&alice);
        let bob_clone = Rc::clone(&bob);

        let pair = Pairing::new(alice, bob);

        let result = pair.end_match(2, 0, 0);
        assert!(result.is_ok());

        let alice = alice_clone.borrow();
        let bob = bob_clone.borrow();

        assert_eq!(alice.game_points, 6);
        assert_eq!(bob.game_points, 0);

        assert_eq!(alice.match_points, 3);
        assert_eq!(bob.match_points, 0);

        assert_eq!(alice.games_played, 2);
        assert_eq!(bob.games_played, 2);

        assert_eq!(alice.matches_played, 1);
        assert_eq!(bob.matches_played, 1);

        assert_eq!(alice.opponents.len(), 1);
        assert_eq!(bob.opponents.len(), 1);
    }

    #[test]
    /// Alice 1-2 Bob
    fn pairing_match_12() {
        let alice = Rc::new(RefCell::new(Player::new("Alice")));
        let bob = Rc::new(RefCell::new(Player::new("Bob")));
        let alice_clone = Rc::clone(&alice);
        let bob_clone = Rc::clone(&bob);

        let pair = Pairing::new(alice, bob);

        let result = pair.end_match(1, 2, 0);
        assert!(result.is_ok());

        let alice = alice_clone.borrow();
        let bob = bob_clone.borrow();

        assert_eq!(alice.game_points, 3);
        assert_eq!(bob.game_points, 6);

        assert_eq!(alice.match_points, 0);
        assert_eq!(bob.match_points, 3);

        assert_eq!(alice.games_played, 3);
        assert_eq!(bob.games_played, 3);

        assert_eq!(alice.matches_played, 1);
        assert_eq!(bob.matches_played, 1);

        assert_eq!(alice.opponents.len(), 1);
        assert_eq!(bob.opponents.len(), 1);
    }

    #[test]
    /// Alice 1-1-1 Bob
    fn pairing_match_111() {
        let alice = Rc::new(RefCell::new(Player::new("Alice")));
        let bob = Rc::new(RefCell::new(Player::new("Bob")));
        let alice_clone = Rc::clone(&alice);
        let bob_clone = Rc::clone(&bob);

        let pair = Pairing::new(alice, bob);

        let result = pair.end_match(1, 1, 1);
        assert!(result.is_ok());

        let alice = alice_clone.borrow();
        let bob = bob_clone.borrow();

        assert_eq!(alice.game_points, 4);
        assert_eq!(bob.game_points, 4);

        assert_eq!(alice.match_points, 1);
        assert_eq!(bob.match_points, 1);

        assert_eq!(alice.games_played, 3);
        assert_eq!(bob.games_played, 3);

        assert_eq!(alice.matches_played, 1);
        assert_eq!(bob.matches_played, 1);

        assert_eq!(alice.opponents.len(), 1);
        assert_eq!(bob.opponents.len(), 1);
    }

    #[test]
    /// Alice 0-0-3 Bob
    fn pairing_match_003() {
        let alice = Rc::new(RefCell::new(Player::new("Alice")));
        let bob = Rc::new(RefCell::new(Player::new("Bob")));
        let alice_clone = Rc::clone(&alice);
        let bob_clone = Rc::clone(&bob);

        let pair = Pairing::new(alice, bob);

        let result = pair.end_match(0, 0, 3);
        assert!(result.is_ok());

        let alice = alice_clone.borrow();
        let bob = bob_clone.borrow();

        assert_eq!(alice.game_points, 3);
        assert_eq!(bob.game_points, 3);

        assert_eq!(alice.match_points, 1);
        assert_eq!(bob.match_points, 1);

        assert_eq!(alice.games_played, 3);
        assert_eq!(bob.games_played, 3);

        assert_eq!(alice.matches_played, 1);
        assert_eq!(bob.matches_played, 1);

        assert_eq!(alice.opponents.len(), 1);
        assert_eq!(bob.opponents.len(), 1);
    }

    #[test]
    /// Alice 4-0 Bob
    fn pairing_match_40() {
        let alice = Rc::new(RefCell::new(Player::new("Alice")));
        let bob = Rc::new(RefCell::new(Player::new("Bob")));
        let alice_clone = Rc::clone(&alice);
        let bob_clone = Rc::clone(&bob);

        let pair = Pairing::new(alice, bob);

        let result = pair.end_match(4, 0, 0);
        assert!(result.is_err());

        let alice = alice_clone.borrow();
        let bob = bob_clone.borrow();

        assert_eq!(alice.game_points, 0);
        assert_eq!(bob.game_points, 0);

        assert_eq!(alice.match_points, 0);
        assert_eq!(bob.match_points, 0);

        assert_eq!(alice.games_played, 0);
        assert_eq!(bob.games_played, 0);

        assert_eq!(alice.matches_played, 0);
        assert_eq!(bob.matches_played, 0);

        assert_eq!(alice.opponents.len(), 1);
        assert_eq!(bob.opponents.len(), 1);
    }

    #[test]
    /// Alice 2-1-2 Bob
    fn pairing_match_212() {
        let alice = Rc::new(RefCell::new(Player::new("Alice")));
        let bob = Rc::new(RefCell::new(Player::new("Bob")));
        let alice_clone = Rc::clone(&alice);
        let bob_clone = Rc::clone(&bob);

        let pair = Pairing::new(alice, bob);

        let result = pair.end_match(2, 1, 2);
        assert!(result.is_err());

        let alice = alice_clone.borrow();
        let bob = bob_clone.borrow();

        assert_eq!(alice.game_points, 0);
        assert_eq!(bob.game_points, 0);

        assert_eq!(alice.match_points, 0);
        assert_eq!(bob.match_points, 0);

        assert_eq!(alice.games_played, 0);
        assert_eq!(bob.games_played, 0);

        assert_eq!(alice.matches_played, 0);
        assert_eq!(bob.matches_played, 0);

        assert_eq!(alice.opponents.len(), 1);
        assert_eq!(bob.opponents.len(), 1);
    }

    #[test]
    /// Alice 0-0-4 Bob
    fn pairing_match_004() {
        let alice = Rc::new(RefCell::new(Player::new("Alice")));
        let bob = Rc::new(RefCell::new(Player::new("Bob")));
        let alice_clone = Rc::clone(&alice);
        let bob_clone = Rc::clone(&bob);

        let pair = Pairing::new(alice, bob);

        let result = pair.end_match(0, 0, 4);
        assert!(result.is_err());

        let alice = alice_clone.borrow();
        let bob = bob_clone.borrow();

        assert_eq!(alice.game_points, 0);
        assert_eq!(bob.game_points, 0);

        assert_eq!(alice.match_points, 0);
        assert_eq!(bob.match_points, 0);

        assert_eq!(alice.games_played, 0);
        assert_eq!(bob.games_played, 0);

        assert_eq!(alice.matches_played, 0);
        assert_eq!(bob.matches_played, 0);

        assert_eq!(alice.opponents.len(), 1);
        assert_eq!(bob.opponents.len(), 1);
    }

    #[test]
    fn tournament_2_players() {
        let mut players = Vec::with_capacity(2);
        let p1 = Rc::new(RefCell::new(Player::new("Player 1")));
        let p2 = Rc::new(RefCell::new(Player::new("Player 2")));
        players.push(p1);
        players.push(p2);

        let mut tourn = Tournament::new(players);
        assert_eq!(tourn.rounds, 1);
        let pairings = tourn.next_round();
        let pair = &pairings.unwrap()[0];
        let uuid = pair.0;
        let home = String::from(&pair.1);
        let away = String::from(&pair.2);

        assert!(
            (home == "Player 1" && away == "Player 2")
                || (home == "Player 2" && away == "Player 1")
        );

        assert!(tourn.end_match(uuid, 2, 1, 0).is_ok());

        assert_eq!(tourn.next_round(), None);

        let players = tourn.ranking();

        let mut bye_count = 0;
        for p in &players {
            if p.borrow().has_bye {
                bye_count += 1;
            }
        }
        assert_eq!(bye_count, 0);

        let winner = players[0].borrow();
        assert_eq!(winner.matches_played, 1);
        assert_eq!(winner.match_points, 3);
        assert_eq!(winner.games_played, 3);
        assert_eq!(winner.game_points, 6);

        let loser = players[1].borrow();
        assert_eq!(loser.matches_played, 1);
        assert_eq!(loser.match_points, 0);
        assert_eq!(loser.games_played, 3);
        assert_eq!(loser.game_points, 3);
    }

    #[test]
    fn tournament_3_players() {
        let mut players = Vec::with_capacity(2);
        let p1 = Rc::new(RefCell::new(Player::new("Player 1")));
        let p2 = Rc::new(RefCell::new(Player::new("Player 2")));
        let p3 = Rc::new(RefCell::new(Player::new("Player 3")));
        players.push(p1);
        players.push(p2);
        players.push(p3);

        let mut tourn = Tournament::new(players);
        assert_eq!(tourn.rounds, 2);

        let re = Regex::new(r"Player (\d)").unwrap();

        while let Some(pairings) = tourn.next_round() {
            for pair in &pairings {
                let uuid = pair.0;

                let home = String::from(&pair.1);
                let away = String::from(&pair.2);

                let home: u32 = re
                    .captures(&home)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .trim()
                    .parse()
                    .unwrap();

                let away: u32 = re
                    .captures(&away)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .trim()
                    .parse()
                    .unwrap();

                let home_score;
                let away_score;
                let drawn = 0;

                if home > away {
                    home_score = 2;
                    away_score = 1;
                } else {
                    home_score = 1;
                    away_score = 2;
                }

                assert!(tourn.end_match(uuid, home_score, away_score, drawn).is_ok());
            }
        }

        let players = tourn.ranking();

        let mut bye_count = 0;
        for p in &players {
            if p.borrow().has_bye {
                bye_count += 1;
            }
        }
        assert_eq!(bye_count, 2);

        let winner = players[0].borrow();
        assert_eq!(winner.name, "Player 3");
        assert_eq!(winner.matches_played, 2);
        assert!(winner.games_played >= 5);
        assert_eq!(winner.match_points, 6);
        assert_eq!(winner.game_points, 12);

        let loser = players[1].borrow();
        assert!(loser.name == "Player 1" || loser.name == "Player 2");
        assert_eq!(loser.matches_played, 2);
        assert!(loser.games_played >= 5);
        assert!(loser.match_points == 0 || loser.match_points == 3);
        assert!(loser.game_points == 3 || loser.game_points == 9);
    }

    #[test]
    fn tournament_4_players() {
        let mut players = Vec::with_capacity(4);

        let p1 = Rc::new(RefCell::new(Player::new("Player 1")));
        let p2 = Rc::new(RefCell::new(Player::new("Player 2")));
        let p3 = Rc::new(RefCell::new(Player::new("Player 3")));
        let p4 = Rc::new(RefCell::new(Player::new("Player 4")));

        players.push(p1);
        players.push(p2);
        players.push(p3);
        players.push(p4);

        let mut tourn = Tournament::new(players);
        assert_eq!(tourn.rounds, 2);

        let re = Regex::new(r"Player (\d)").unwrap();

        while let Some(pairings) = tourn.next_round() {
            for pair in &pairings {
                let uuid = pair.0;

                let home = String::from(&pair.1);
                let away = String::from(&pair.2);

                let home: u32 = re
                    .captures(&home)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .trim()
                    .parse()
                    .unwrap();

                let away: u32 = re
                    .captures(&away)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .trim()
                    .parse()
                    .unwrap();

                let home_score;
                let away_score;
                let drawn = 0;

                if home > away {
                    home_score = 2;
                    away_score = 1;
                } else {
                    home_score = 1;
                    away_score = 2;
                }

                assert!(tourn.end_match(uuid, home_score, away_score, drawn).is_ok());
            }
        }

        let players = tourn.ranking();

        let mut bye_count = 0;
        for p in &players {
            if p.borrow().has_bye {
                bye_count += 1;
            }
        }
        assert_eq!(bye_count, 0);

        let p1 = players[0].borrow();
        assert_eq!(p1.name, "Player 4");
        assert_eq!(p1.matches_played, 2);
        assert_eq!(p1.games_played, 6);
        assert_eq!(p1.match_points, 6);
        assert_eq!(p1.game_points, 12);

        let p2 = players[1].borrow();
        assert!(p2.name == "Player 3" || p2.name == "Player 2");
        assert_eq!(p2.matches_played, 2);
        assert_eq!(p2.games_played, 6);
        assert_eq!(p2.match_points, 3);
        assert_eq!(p2.game_points, 9);

        let p3 = players[2].borrow();
        assert!(p3.name == "Player 3" || p3.name == "Player 2");
        assert_eq!(p3.matches_played, 2);
        assert_eq!(p3.games_played, 6);
        assert_eq!(p3.match_points, 3);
        assert_eq!(p3.game_points, 9);

        let p4 = players[3].borrow();
        assert_eq!(p4.name, "Player 1");
        assert_eq!(p4.matches_played, 2);
        assert_eq!(p4.games_played, 6);
        assert_eq!(p4.match_points, 0);
        assert_eq!(p4.game_points, 6);
    }

    #[test]
    fn tournament_8_players() {
        let mut players = Vec::with_capacity(8);

        for i in 1..9 {
            let p = Rc::new(RefCell::new(Player::new(format!("Player {}", i).as_str())));
            players.push(p);
        }

        let mut tourn = Tournament::new(players);
        assert_eq!(tourn.rounds, 3);

        let re = Regex::new(r"Player (\d)").unwrap();

        while let Some(pairings) = tourn.next_round() {
            for pair in &pairings {
                let uuid = pair.0;

                let home = String::from(&pair.1);
                let away = String::from(&pair.2);

                let home: u32 = re
                    .captures(&home)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .trim()
                    .parse()
                    .unwrap();

                let away: u32 = re
                    .captures(&away)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .trim()
                    .parse()
                    .unwrap();

                let home_score;
                let away_score;
                let drawn = 0;

                if home > away {
                    home_score = 2;
                    away_score = 1;
                } else {
                    home_score = 1;
                    away_score = 2;
                }

                assert!(tourn.end_match(uuid, home_score, away_score, drawn).is_ok());
            }
        }

        let players = tourn.ranking();

        let p1 = players[0].borrow();
        assert_eq!(p1.name, "Player 8");
        assert_eq!(p1.matches_played, 3);
        assert_eq!(p1.games_played, 9);
        assert_eq!(p1.match_points, 9);
        assert_eq!(p1.game_points, 18);

        let p2 = players[7].borrow();
        assert_eq!(p2.matches_played, 3);
        assert_eq!(p2.games_played, 9);
        assert_eq!(p2.match_points, 0);
        assert_eq!(p2.game_points, 9);

        let mut bye_count = 0;
        for p in &players {
            if p.borrow().has_bye {
                bye_count += 1;
            }
        }
        assert_eq!(bye_count, 0);
    }

    #[test]
    fn tournament_13_players() {
        let mut players = Vec::with_capacity(13);

        for i in 1..14 {
            let p = Rc::new(RefCell::new(Player::new(format!("Player {}", i).as_str())));
            players.push(p);
        }

        let mut tourn = Tournament::new(players);
        assert_eq!(tourn.rounds, 4);

        let re = Regex::new(r"Player (\d+)").unwrap();

        while let Some(pairings) = tourn.next_round() {
            for pair in &pairings {
                let uuid = pair.0;

                let home = String::from(&pair.1);
                let away = String::from(&pair.2);

                let home: u32 = re
                    .captures(&home)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .trim()
                    .parse()
                    .unwrap();

                let away: u32 = re
                    .captures(&away)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .trim()
                    .parse()
                    .unwrap();

                let home_score;
                let away_score;
                let drawn = 0;

                if home > away {
                    home_score = 2;
                    away_score = 1;
                } else {
                    home_score = 1;
                    away_score = 2;
                }

                assert!(tourn.end_match(uuid, home_score, away_score, drawn).is_ok());
            }
        }

        let players = tourn.ranking();

        let mut bye_count = 0;
        for p in &players {
            if p.borrow().has_bye {
                bye_count += 1;
            }
        }
        assert_eq!(bye_count, 4);

        let winner = players[0].borrow();
        assert_eq!(winner.name, "Player 13");
        assert_eq!(winner.matches_played, 4);
        assert!(winner.games_played >= 11);
        assert_eq!(winner.match_points, 12);
        assert_eq!(winner.game_points, 24);

        let loser = players[12].borrow();
        assert_eq!(loser.matches_played, 4);
        assert!(loser.games_played >= 11);
        assert!(loser.match_points == 0 || loser.match_points == 3);
        assert!(loser.game_points == 12 || loser.game_points == 15);
    }

    #[test]
    fn tournament_60_players() {
        let mut players = Vec::with_capacity(60);

        for i in 1..61 {
            let p = Rc::new(RefCell::new(Player::new(format!("Player {}", i).as_str())));
            players.push(p);
        }

        let mut tourn = Tournament::new(players);
        assert_eq!(tourn.rounds, 6);

        let re = Regex::new(r"Player (\d+)").unwrap();

        while let Some(pairings) = tourn.next_round() {
            for pair in &pairings {
                let uuid = pair.0;

                let home = String::from(&pair.1);
                let away = String::from(&pair.2);

                let home: u32 = re
                    .captures(&home)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .trim()
                    .parse()
                    .unwrap();

                let away: u32 = re
                    .captures(&away)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .trim()
                    .parse()
                    .unwrap();

                let home_score;
                let away_score;
                let drawn = 0;

                if home > away {
                    home_score = 2;
                    away_score = 1;
                } else {
                    home_score = 1;
                    away_score = 2;
                }

                assert!(tourn.end_match(uuid, home_score, away_score, drawn).is_ok());
            }
        }

        let players = tourn.ranking();

        let mut bye_count = 0;
        for p in &players {
            if p.borrow().has_bye {
                bye_count += 1;
            }
        }
        assert_eq!(bye_count, 0);

        let winner = players[0].borrow();
        assert_eq!(winner.name, "Player 60");
        assert_eq!(winner.matches_played, 6);
        assert!(winner.games_played >= 17);
        assert_eq!(winner.match_points, 18);
        assert_eq!(winner.game_points, 36);

        let loser = players[59].borrow();
        assert_eq!(loser.matches_played, 6);
        assert!(loser.games_played >= 17);
        assert!(loser.match_points == 0 || loser.match_points == 3);
        assert!(loser.game_points == 18 || loser.game_points == 21);
    }
}
