use crate::card::*;
use std::{collections::{hash_map::Entry, HashMap}, fmt::Display, hash::{Hash, Hasher}, ops::{BitOr, BitOrAssign}};
use itertools::Itertools;


/// Slightly goofy bit representation of hand
/// Engineered so (hand1 = hand2) <=> (score(hand1) <=> score(hand2)) [if there are 5 cards in the hand]
/// Low 39 bits for ranks:
/// - count the number of occurences of each rank (0-4)
/// - 3 bits per rank * 13 ranks = 39 bits
/// -   Lowest rank in lowest 3 bits (e.g. Two in 0..=2)
/// -   Ace in 36..=38
/// 
/// Leading bit is 1 if the hand has a flush (i.e. 5 cards of the same suit)
/// 
/// Following 13 bits indicate which ranks have the flush suit:
/// - i.e. bit 62 is set if the hand has a flush with an Ace (bit 63 is discriminant)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Hand(u64);

impl Hand {
    pub const EMPTY: Hand = Hand(0);
    const SUIT_OFFSET: u64 = 50;

    pub fn new(cards: &Vec<Card>) -> Hand {
        let mut val = 0;
        let mut suit_counts: [usize; 4] = [0; 4];
        for card in cards {
            val += 1 << (usize::from(card.rank) * 3);
            suit_counts[usize::from(card.suit)] += 1;
        }
        if let Some((flush_suit, _)) = suit_counts.iter().find_position(|&&x| x >= 5) {
            val |= 1 << 63; // Set flush bit

            for card in cards {
                if card.suit == Suit::try_from(flush_suit).unwrap() {
                    val |= 1 << (usize::from(card.rank) as u64 + Hand::SUIT_OFFSET);
                }
            }
        } 

        Hand(val)
    }

    /// Get all combinations of n cards, best sorted first
    pub fn get_hand_combos(n: usize) -> Vec<Hand> {
        Card::get_deck()
            .into_iter()
            .rev()
            .combinations(n)
            .map(|combo| Hand::new(&combo))
            .collect()
    } 

    pub fn from_straight_flush(high_rank: Rank) -> Hand {
        let mut hand = Hand::EMPTY;
        hand.0 |= 1 << 63; // Set flush bit
        if high_rank == Rank::Five {
            hand.0 |= 0b1111 << Hand::SUIT_OFFSET;
            hand.0 |= 1 << 62;
            
            hand.0 |= 0b001001001001;
            hand.0 |= 1 << (usize::from(Rank::Ace) * 3);
            return hand;
        }
        
        let low_rank_offset = usize::from(high_rank) as u64 - 4;
        hand.0 |= 0b11111 << (low_rank_offset + Hand::SUIT_OFFSET);
        hand.0 |= 0b001001001001001 << low_rank_offset * 3;
        return hand;
    }

    pub fn from_straight(high_rank: Rank) -> Hand {
        let mut hand = Hand::EMPTY;
        if high_rank == Rank::Five {
            hand.0 |= 0b001001001001;
            hand.0 |= 1 << (usize::from(Rank::Ace) * 3);
            return hand;
        }
        
        let low_rank_offset = usize::from(high_rank) as u64 - 4;
        hand.0 |= 0b001001001001001 << low_rank_offset*3;
        return hand;
    }

    pub fn contains_rank(&self, rank: Rank) -> bool {
        ((0b111 << (usize::from(rank) * 3)) & self.0) != 0
    }

    fn add_rank(&mut self, rank: Rank) {
        debug_assert!(self.count_rank(rank) < 4);
        self.0 += 1 << (usize::from(rank) * 3);
    }

    fn take_rank(&mut self, rank: Rank) {
        debug_assert!(self.contains_rank(rank));
        self.0 -= 1 << (usize::from(rank) * 3);
    }

    fn add_n_rank(&mut self, rank: Rank, n: u64) {
        debug_assert!(self.count_rank(rank) + n <= 4);
        self.0 += n << (usize::from(rank) * 3)
    }

    pub fn count_rank(&self, rank: Rank) -> u64 {
        ((0b111 << (usize::from(rank) * 3)) & self.0) >> (usize::from(rank) * 3)
    }

    pub fn is_flush(&self) -> bool {
        self.0 & (1 << 63) != 0
    }

    fn is_in_flush(self, rank: Rank) -> bool {
        (self.0 & (1 << (usize::from(rank) as u64 + Hand::SUIT_OFFSET))) != 0 
    }

    fn from_rank_as_flush(rank: Rank) -> Hand {
        let mut hand = Hand::EMPTY;
        hand.0 |= 1 << 63; // Set flush bit
        hand.0 |= 1 << (usize::from(rank) as u64 + Hand::SUIT_OFFSET);
        hand.0 |= 1 << (usize::from(rank) * 3);
        return hand;
    }

    fn from_n_rank(rank: Rank, n: u64) -> Hand {
        debug_assert!(n <= 4);
        Hand(n << (usize::from(rank) * 3))
    }
    /// Get all combinations of n ranks as flush
    fn flush_combos() -> Vec<Hand> {
        Rank::ALL_RANKS
            .iter()
            .map(|&rank| Hand::from_rank_as_flush(rank))
            .rev()
            .combinations(5)
            .map(|combo| combo.into_iter().fold(Hand::EMPTY, |acc, hand| {
                acc | hand
            }))
            .collect()
    }

}

impl Hash for Hand {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
    }
}

impl BitOr for Hand {
    type Output = Hand;

    fn bitor(self, other: Hand) -> Hand {
        Hand(self.0 | other.0)
    }
}

impl BitOrAssign for Hand {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in Rank::ALL_RANKS {
            for _ in 0..self.count_rank(rank) {
                write!(f, "[{}{}] ",
                    if self.is_in_flush(rank) {"f"} else {""},
                    rank
                    )?
            }
        }
        Ok(())
    }
}   

/// Scores only 5 length
fn score_straight_flush(scores: &mut HashMap<Hand, u64>, offset: u64) -> u64 {
    let mut score: u64 = offset;
    for high_rank in (usize::from(Rank::Five)..=usize::from(Rank::Ace)).rev() {
        let hand = Hand::from_straight_flush(high_rank.try_into().unwrap());
        if let Entry::Vacant(v) = scores.entry(hand) {
                v.insert(score);
                score += 1;
            }
    }

    return score;
}

/// Doesn't need to consider flushes because:
/// - they are not possible with 4 of a kind (with at most 7 cards)
/// - all others are worth less than a flush anyway
fn score_n_of_a_kind(scores: &mut HashMap<Hand, u64>, offset: u64, n: u64) -> u64 {
    let mut score: u64 = offset;
    for set_rank in Rank::ALL_RANKS.iter().rev() {
        for kickers in Hand::get_hand_combos(5 - n as usize) {
            if kickers.contains_rank(*set_rank) {
                continue;
            }
            let mut hand = Hand::from_n_rank(*set_rank, n as u64);
            hand |= kickers;

            if let Entry::Vacant(v) = scores.entry(hand) {
                v.insert(score);
                score += 1;
            }
        }
    }
    return score;
}

/// Also doesn't need to consider flushes it's impossible with 7 cards
fn score_full_house(scores: &mut HashMap<Hand, u64>, offset: u64) -> u64 {
    let mut score: u64 = offset;
    for three_rank in Rank::ALL_RANKS.iter().rev() {
        for pair_rank in Rank::ALL_RANKS.iter().rev() {
            if three_rank == pair_rank {
                continue;
            }
            let mut hand = Hand::from_n_rank(*three_rank, 3);
            hand.add_n_rank(*pair_rank, 2);
            if let Entry::Vacant(v) = scores.entry(hand) {
                v.insert(score);
                score += 1;
            }
        }
    }
    return score;
}


fn score_flush(scores: &mut HashMap<Hand, u64>, offset: u64) -> u64 {
    let mut score: u64 = offset;
    for hand in Hand::flush_combos() {
        if let Entry::Vacant(v) = scores.entry(hand) {
            v.insert(score);
            score += 1;
        }
    }
    return score;
}

fn score_straight(scores: &mut HashMap<Hand, u64>, offset: u64) -> u64 {
    let mut score: u64 = offset;
    for high_rank in (usize::from(Rank::Five)..=usize::from(Rank::Ace)).rev() {
        let hand = Hand::from_straight(high_rank.try_into().unwrap());
        if let Entry::Vacant(v) = scores.entry(hand) {
            v.insert(score);
            score += 1;
        }
    }
    return score;
}

fn score_two_pair(scores: &mut HashMap<Hand, u64>, offset: u64) -> u64 {
    let mut score: u64 = offset;
    for high_pair in (usize::from(Rank::Three)..=usize::from(Rank::Ace)).rev() {
        for low_pair in (usize::from(Rank::Two)..high_pair).rev() {
            let mut hand = Hand::from_n_rank(high_pair.try_into().unwrap(), 2);
            hand.add_n_rank(low_pair.try_into().unwrap(), 2);
            
            for kicker in Rank::ALL_RANKS.iter().rev() {
                if hand.contains_rank(*kicker) {
                    continue;
                }
                hand.add_rank(*kicker);
                
                if let Entry::Vacant(v) = scores.entry(hand) {
                    v.insert(score);
                    score += 1;
                }
                hand.take_rank(*kicker);
            }
        }
    }
    return score;
}

fn score_high_card(scores: &mut HashMap<Hand, u64>, offset: u64) -> u64 {
    let mut score: u64 = offset;
    for hand in Hand::get_hand_combos(5) {
        if let Entry::Vacant(v) = scores.entry(hand) {
            v.insert(score);
            score += 1;
        }
    }
    return score;
}

pub fn create_score_table() -> (HashMap<Hand, u64>, u64) {
    let mut scores: HashMap<Hand, u64> = HashMap::new();
    let mut score: u64 = 0;
    score = score_straight_flush(&mut scores, score);
    score = score_n_of_a_kind(&mut scores, score, 4);
    score = score_full_house(&mut scores, score);
    score = score_flush(&mut scores, score);
    score = score_straight(&mut scores, score);
    score = score_n_of_a_kind(&mut scores, score, 3);
    score = score_two_pair(&mut scores, score);
    score = score_n_of_a_kind(&mut scores, score, 2);
    score = score_high_card(&mut scores, score);

    return (scores, score);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let cards: Vec<Card> = vec![Rank::Two, Rank::Three, Rank::Four, Rank::Ace].iter().map(|rank| Card::new(*rank, Suit::Hearts)).collect();
        let hand = Hand::new(&cards);
        for card in cards {
            assert!(hand.contains_rank(card.rank));
        }
    }

    #[test]
    fn test_flush_makers() {
        for high_rank in (usize::from(Rank::Five)..=usize::from(Rank::Ace)).rev() {
            let hand1 = Hand::from_straight_flush(high_rank.try_into().unwrap());
            let ranks: Vec<Rank> = Rank::ALL_RANKS.iter().rev().cycle().skip(usize::from(Rank::Ace) - high_rank).take(5).copied().collect();
            let mut hand2 = Hand::EMPTY;
            for rank in ranks {
                hand2 |= Hand::from_rank_as_flush(rank);
            }
            assert_eq!(hand2, hand1);
        }
    }   

    #[test]
    fn test_makers() {
        for hand in Hand::get_hand_combos(5) {
            let mut hand2 = Hand::EMPTY;
            if hand.is_flush() {
                for rank in Rank::ALL_RANKS {
                    if hand.is_in_flush(rank) {
                        assert!(hand.count_rank(rank) > 0);
                        hand2 |= Hand::from_rank_as_flush(rank);
                        hand2.add_n_rank(rank, hand.count_rank(rank)- 1);
                    } else {
                        hand2.add_n_rank(rank, hand.count_rank(rank));
                    }
                }
            } else  {
                for rank in Rank::ALL_RANKS {
                    assert!(!hand.is_in_flush(rank));
                    hand2.add_n_rank(rank, hand.count_rank(rank));
                }
            }
            assert_eq!(hand, hand2);
        }
    }

    #[test]
    fn test_add_and_remove() {
        for hand in Hand::get_hand_combos(5) {
            let mut hand2  = hand.clone();
            for rank in Rank::ALL_RANKS {
                if !hand2.contains_rank(rank) {
                    hand2.add_rank(rank);
                    hand2.take_rank(rank);
                    assert_eq!(hand, hand2);
                }         
            }
        }
    }

    #[test]
    fn test_score_table() {
        // confirm that the no. of distinct hands in each category matches
        let mut scores: HashMap<Hand, u64> = HashMap::new();
        assert_eq!(score_straight_flush(&mut scores, 0), 10);
        assert_eq!(score_n_of_a_kind(&mut scores, 0, 4), 156);
        assert_eq!(score_full_house(&mut scores, 0), 156);
        assert_eq!(score_flush(&mut scores, 0), 1277);
        assert_eq!(score_straight(&mut scores, 0), 10);
        assert_eq!(score_n_of_a_kind(&mut scores, 0, 3), 858);
        assert_eq!(score_two_pair(&mut scores, 0), 858);
        assert_eq!(score_n_of_a_kind(&mut scores, 0,2), 2860);
        assert_eq!(score_high_card(&mut scores, 0), 1277);

    }
}