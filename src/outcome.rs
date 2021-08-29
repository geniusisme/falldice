//use common::*;
//use enum_map as em;

#[macro_export]
macro_rules! outcome {
    ($Score:ty, $($variant:ident,)+) => {

//////////////////////////////////////////

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, enum_map::Enum)]
#[repr(u8)]
pub enum Facet {
    $($variant,)+
}

pub use self::Facet::*;

pub type Scores = enum_map::EnumMap<self::Facet, $Score>;

#[derive(Clone)]
pub struct Outcome {
    pub probability: Real,
    pub scores: Scores,
}

impl Outcome {
    pub fn new() -> Outcome {
        Outcome { probability: 1.0, scores: Scores::default() }
    }

    pub fn from_prob_scores_iter(
        prob: Real, 
        scores_iter: impl Iterator<Item=(self::Facet, $Score)>) 
    -> Outcome {
        let mut scores = Scores::default();
        scores.extend(scores_iter);
        Outcome {
            probability: prob,
            scores: scores
        }
    }
}

pub fn update(what: &mut Scores, with: &Scores, f: impl Fn(&mut $Score, &$Score)) {
    what.values_mut().zip(with.values()).for_each(|(lhs, rhs)| {
        f(lhs, rhs);
    });
}

/*fn combine_outcomes(lhs: &Outcome, rhs: &Outcome) -> Outcome {
    let mut combined_scores = Scores::default();
    combined_scores.extend(lhs.scores.iter().zip(rhs.scores.iter()).map(|(l,r)| (l.0, l.1 + r.1)));
    //let mut combined_scores = lhs.scores;
    //combined_scores.extend(rhs.scores);
    Outcome {
        probability: lhs.probability * rhs.probability,        
        scores: combined_scores,
    }
}*/

//////////////////////////////////////////

    };
}
