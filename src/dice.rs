use common::*;
use cartesian_fold::IterExt;
use outcome;

outcome! {
    Score,
    Damage,
    Shred,
    Skill,
    Bottle,
    Star,
    Explosion,
    BlackBlank,
    GreenBlank,
    YellowBlank,
    Armor,
    Crit,
    Action,
    Hit,
    Miss,
}

pub enum Die {
    White,
    Red,
    Yellow,
    Green,
    Black,
    Blue,
}

pub use Die::*;

pub fn outcomes(dice: &[Die]) -> impl Iterator<Item=Outcome> + '_ {
    dice.iter()
        .map(|die| match die {
            White => white,
            Red => red,
            Yellow => yellow,
            Green => green,
            Black => black,
            Blue => blue,
        }.iter())
        .cartesian_collections(|iter|
            iter.fold(Outcome::new(), |mut acc, face| {
                acc.probability *= face.0;
                face.1.iter().fold(acc, |mut acc, score| {
                    acc.scores[score.0] += score.1;
                    acc
                })
            })
        )

}

pub type Face = &'static [(Facet, Score)];

pub type DieDesc = &'static [(Real, Face)];

macro_rules! make_die {

    [$faces_cnt:literal, $($faces:literal =>  $($desc:expr), +;) +] => {
        &[$(($faces as Real / $faces_cnt as Real, &[$($desc), +])), +]
    };
}

pub const white: DieDesc = make_die![
    20,
    2 => (Action, 1), (Hit, 1);
    3 => (Crit, 1), (Hit, 1);
    1 => (Skill, 2);
    1 => (Skill, 3);
    1 => (Skill, 4);
    1 => (Skill, 5);
    2 => (Skill, 6);
    2 => (Skill, 7);
    2 => (Skill, 8);
    1 => (Skill, 9);
    1 => (Skill, 10);
    2 => (Miss, 1);
    1 => (Miss, 1), (Action, 1);
];

pub const red: DieDesc = make_die![
    12,
    4 => (Armor, 1);
    3 => (Armor, 2);
    3 => (Armor, 3);
    2 => (Armor, 4);
];

pub const black: DieDesc = make_die![
    12,
    5 =>  (Damage, 1);
    1 =>  (Damage, 2);
    3 =>  (BlackBlank, 1);
    1 =>  (Bottle, 1);
    1 =>  (Shred, 1);
    1 =>  (Skill, -1);
];

pub const green: DieDesc = make_die![
    12,
    4 =>  (Skill, -2), (GreenBlank, 1);
    2 =>  (Skill, -2);
    1 =>  (Skill, -3);
    2 =>  (Shred, 1);
    2 =>  (Damage, 1);
    1 =>  (Bottle, 1);
];

pub const yellow: DieDesc = make_die![
    12,
    5 =>  (Shred, 1);
    1 =>  (Shred, 2);
    2 =>  (YellowBlank, 1);
    1 =>  (Bottle, 1);
    2 =>  (Skill, -1);
    1 =>  (Damage, 1);
];

pub const blue: DieDesc = make_die![
    12,
    4 =>  (Bottle, 1);
    2 =>  (Bottle, 2);
    1 =>  (Bottle, 1), (Star, 1);
    2 =>  (Star, 1);
    1 =>  (Star, 2);
    2 =>  (Explosion, 1);
];
