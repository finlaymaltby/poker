mod card;
mod hand;
use card::*;
use itertools::Itertools;
use hand::*;

use std::{collections::HashMap, sync::LazyLock, time::Instant};
use rand::{seq::IteratorRandom, rng};

static SCORES: LazyLock<HashMap<Hand, u64>> = LazyLock::new(|| hand::create_score_table().0);

fn get_best_score(pair: &(Card, Card), community: &Vec<Card>) -> u64 {
    community.clone()
        .into_iter()
        .chain(std::iter::once(pair.0))
        .chain(std::iter::once(pair.1))
        .combinations(5)
        .map(|cards|  Hand::new(&cards))
        .map(|hand| *SCORES.get(&hand).unwrap())
        .min()
        .unwrap()
}


/// exhaustive search is manageable with at least the flop on the board
/// returns (win_count, lose_count)
fn eval_with_community(community: Vec<Card>, pair: &(Card, Card)) -> (usize, usize) {
    let mut win_count: usize = 0;
    let mut lose_count: usize = 0;

    let mut deck: Vec<Card> = Card::get_deck();
    deck.retain(|card| !community.contains(card) && *card != pair.0 && *card != pair.1);

    let evil_pairs: Vec<(Card, Card)> = deck
        .iter()
        .copied()
        .tuple_combinations()
        .collect();

    let mut community = community;
    let n = community.len();

    for remainder in deck.iter().copied().combinations(5-n) {
        community.append(&mut remainder.clone());

        let my_score = get_best_score(pair, &community);

        for evil_pair in &evil_pairs {
            // Skip if evil_pair contains turn or river
            if remainder.contains(&evil_pair.0) || remainder.contains(&evil_pair.1) {
                continue;
            }
            if my_score < get_best_score(evil_pair, &community) {
                win_count += 1;
            } else {
                lose_count += 1;
            }
        }
        community.truncate(n);
    }
    (win_count, lose_count)
}

/// not currently feasible to do an exhaustive search with just the hand
/// so a monte carlo random search is implemented
fn eval_hand_monte_carlo(pair: &(Card, Card), n: usize) -> (usize, usize) {
    let mut win_count: usize = 0;
    let mut lose_count: usize = 0;

    let mut deck: Vec<Card> = Card::get_deck();
    deck.retain(|card| *card != pair.0 && *card != pair.1);

    let mut rng = rng();


    for community in deck.iter().copied().combinations(5).choose_multiple(&mut rng, n) {
        
        let score = get_best_score(pair, &community);
        for evil_pair in deck.iter().copied().tuple_combinations::<(Card,Card)>() {
            
            if community.contains(&evil_pair.0) || community.contains(&evil_pair.1) {
                continue;
            }

            if score < get_best_score(&evil_pair, &community) {
                win_count += 1;
            } else {
                lose_count += 1;
            }
        }
    }
    return (win_count, lose_count)
}

fn main() {
    let community = vec![Card::new(Rank::Ace, Suit::Hearts), 
                                        Card::new(Rank::King, Suit::Hearts), 
                                        Card::new(Rank::Four, Suit::Spades)];
                    
    let my_hand = (Card::new(Rank::Two, Suit::Hearts), Card::new(Rank::Three, Suit::Hearts));


    let (win, lose) = eval_with_community(community, &my_hand);

    println!("{}: {} {}", (win as f64)/((win+lose) as f64), win, lose)
    
}
