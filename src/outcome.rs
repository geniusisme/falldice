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

#[allow(unused)]
pub fn new_scores(facets: &[(Facet, $Score)]) -> Scores {
  let mut res = Scores::default();
  res.extend(facets.into_iter().cloned());
  res
}

#[derive(Clone)]
pub struct Outcome {
    pub probability: Real,
    pub scores: Scores,
}

impl Outcome {
    #[allow(unused)]
    pub fn new() -> Outcome {
        Outcome { probability: 1.0, scores: Scores::default() }
    }
}

pub fn update(what: &mut Scores, with: &Scores, f: impl Fn(&mut $Score, &$Score)) {
    what.values_mut().zip(with.values()).for_each(|(lhs, rhs)| {
        f(lhs, rhs);
    });
}

impl Default for Outcome {
    fn default() -> Self {
        Outcome { probability: 1.0, scores: Default::default() }
    }
}

//////////////////////////////////////////

    };
}
