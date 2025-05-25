use std::{fmt::{Display, Formatter}, sync::LazyLock};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}
 
impl Rank {
    pub const ALL_RANKS: [Rank; 13] = [
        Rank::Two,   Rank::Three, Rank::Four,  Rank::Five,  Rank::Six,
        Rank::Seven, Rank::Eight, Rank::Nine,  Rank::Ten,   Rank::Jack,
        Rank::Queen, Rank::King,  Rank::Ace,
    ];
}

impl From<Rank> for usize {
    fn from(rank: Rank) -> Self {
        match rank {
            Rank::Two => 0,
            Rank::Three => 1,
            Rank::Four => 2,
            Rank::Five => 3,
            Rank::Six => 4,
            Rank::Seven => 5,
            Rank::Eight => 6,
            Rank::Nine => 7,
            Rank::Ten => 8,
            Rank::Jack => 9,
            Rank::Queen => 10,
            Rank::King => 11,
            Rank::Ace => 12,
        }
    }
}

impl TryFrom<usize> for Rank {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Rank::Two),
            1 => Ok(Rank::Three),
            2 => Ok(Rank::Four),
            3 => Ok(Rank::Five),
            4 => Ok(Rank::Six),
            5 => Ok(Rank::Seven),
            6 => Ok(Rank::Eight),
            7 => Ok(Rank::Nine),
            8 => Ok(Rank::Ten),
            9 => Ok(Rank::Jack),
            10 => Ok(Rank::Queen),
            11 => Ok(Rank::King),
            12 => Ok(Rank::Ace),
            _ => Err("Invalid rank value"),
        }
    }
}

impl Display for Rank {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "10",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
            Rank::Ace => "A",
        })
    }
}


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

impl Suit {
    pub const ALL_SUITS: [Suit; 4] = [
        Suit::Hearts,
        Suit::Diamonds,
        Suit::Clubs,
        Suit::Spades,
    ];
}

impl From<Suit> for usize {
    fn from(suit: Suit) -> Self {
        match suit {
            Suit::Hearts => 0,
            Suit::Diamonds => 1,
            Suit::Clubs => 2,
            Suit::Spades => 3,
        }
    }
}

impl TryFrom<usize> for Suit {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Suit::Hearts),
            1 => Ok(Suit::Diamonds),
            2 => Ok(Suit::Clubs),
            3 => Ok(Suit::Spades),
            _ => panic!("Invalid suit value"),
        }
    }
}

impl Display for Suit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Suit::Hearts => "♥",
            Suit::Diamonds => "♦",
            Suit::Clubs => "♣",
            Suit::Spades => "♠",
        })
    }
}


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    /// Deck of cards, ordered by rank then suit
    const DECK: LazyLock<Vec<Card>> = LazyLock::new(|| {
        let mut cards = Vec::new();
        for rank in Rank::ALL_RANKS {
            for suit in Suit::ALL_SUITS {
                cards.push(Card {
                    rank: rank,
                    suit: suit,
                });
            }
        }
        cards
    });

    pub fn new(rank: Rank, suit: Suit) -> Self {
        Card { rank, suit }
    }

    pub fn get_deck() -> Vec<Card> {
        (*Self::DECK).clone()
    }
}

impl From<Card> for usize {
    fn from(card: Card) -> Self {
        (usize::from(card.rank) * 4) + usize::from(card.suit)
    }
}

impl TryFrom<usize> for Card {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value > 51 {
            return Err("Invalid card value");
        }
        let rank = Rank::try_from(value / 4)?;
        let suit = Suit::try_from(value % 4)?;
        Ok(Card { rank, suit })
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

