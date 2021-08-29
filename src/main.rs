#![allow(non_upper_case_globals)]

extern crate enum_map;

use std::iter::Iterator;
use std::convert::TryInto;
use dice::{Facet, Scores, Die};

mod rollup;
mod dice;
mod attack;
mod outcome;
mod cartesian_fold;
mod effects;

mod common {
    pub type Real = f64;
    pub type Score = i8;
    pub type PositiveScore = std::num::NonZeroI8;
}

use common::*;

fn main() {

    let damage_on_bottle = &effects::ExchangeScoreOnHit {
        give: (dice::Bottle, 1.try_into().unwrap()),
        take: (dice::Damage, 1.try_into().unwrap()),
        times: None
    };

    let shred_on_bottle = &effects::ExchangeScoreOnHit {
        give: (dice::Bottle, 1.try_into().unwrap()),
        take: (dice::Shred, 1.try_into().unwrap()),
        times: None
    };

    let damage_on_star = &effects::ExchangeScoreOnHit {
        give: (dice::Star, 1.try_into().unwrap()),
        take: (dice::Damage, 1.try_into().unwrap()),
        times: None
    };

    println!("result: {:?}", attack::Disposition {
        dice: vec![dice::Red, dice::White, dice::Black, dice::Black, dice::Green, dice::Black, dice::Yellow, dice::Yellow ],
        characteristics: attack::Characteristics {
            base_score: new_scores(&[
                (dice::Damage, 3) ,

                //(dice::Shred, 1),
            ]),
            required_skill: 9,
            soft_armor: 4,
            hard_armor: 0,

        },
        //effects: vec![&damage_on_bottle],
        ..Default::default()
    }.average_scores());


    println!("sniper: {:?}", attack::Disposition {
        dice: vec![dice::Red, dice::White, dice::Green, dice::Green, dice::Blue],
        characteristics: attack::Characteristics {
            base_score: new_scores(&[
                (dice::Damage, 2) ,
                //(dice::Shred, 1),
            ]),
            required_skill: 9,
            soft_armor: 5,
            hard_armor: 1,
        },

        effects: vec![shred_on_bottle, damage_on_star],
        ..Default::default()
    }.average_scores());

    println!("result: {:?}", attack::Disposition {
        dice: vec![dice::Red, dice::White, dice::Black, dice::Black, dice::Yellow, dice::Green, dice::Green],
        characteristics: attack::Characteristics {
            base_score: new_scores(&[
                (dice::Damage, 1) ,
                //(dice::Shred, 1),
            ]),
            required_skill: 9,
            soft_armor: 2,
            hard_armor: 1,
        },
        ..Default::default()
    }.average_scores());

    println!("result: {:?}", attack::Disposition {
        dice: vec![dice::Red, dice::White, dice::Black, dice::Black, dice::Yellow, dice::Green, dice::Green],
        characteristics: attack::Characteristics {
            base_score: new_scores(&[
                (dice::Damage, 1) ,
                //(dice::Shred, 1),
            ]),
            required_skill: 7,
            soft_armor: 2,
            hard_armor: 1,
        },
        ..Default::default()
    }.average_scores());


}

pub fn broke_legs_on_star(attack_s: &mut attack::Scores, dice_s: &mut dice::Scores) {
    if dice_s[dice::Star] > 0 {
        dice_s[dice::Star] -= 1;
        attack_s[attack::BrokenLegs] += 1.0;
    }
}

pub fn damage_on_blank(scores: &mut Scores) {
    if let Some(blank) = scores.iter()
        .filter_map(|score| if *score.1 > 0 { Some(score.0) } else { None })
        .find(|&icon| match icon {
            dice::GreenBlank | dice::YellowBlank | dice::BlackBlank => true,
            _ => false,
        }) {
        scores[blank] -= 1;
        scores[dice::Damage] += 1;
    }
}

fn new_scores(facets: &[(Facet, Score)]) -> Scores {
    let mut res = Scores::default();
    res.extend(facets.into_iter().cloned());
    res
}

