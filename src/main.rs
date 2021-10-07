#![allow(non_upper_case_globals)]

extern crate enum_map;

use std::convert::TryInto;

mod attack;
mod cartesian_fold;
mod dice;
mod effects;
mod outcome;
mod rollup;

mod common {
  pub type Real = f64;
  pub type Score = i8;
  pub type PositiveScore = std::num::NonZeroI8;
}

fn main() {
  #![allow(unused)]

  let damage_on_bottle = &effects::ExchangeScoreOnHit {
    give: (dice::Bottle, 1.try_into().unwrap()),
    take: (dice::Damage, 1.try_into().unwrap()),
    times: None,
  };

  let shred_on_bottle = &effects::ExchangeScoreOnHit {
    give: (dice::Bottle, 1.try_into().unwrap()),
    take: (dice::Shred, 1.try_into().unwrap()),
    times: None,
  };

  let damage_on_star = &effects::ExchangeScoreOnHit {
    give: (dice::Star, 1.try_into().unwrap()),
    take: (dice::Damage, 1.try_into().unwrap()),
    times: None,
  };

  let ignore_armor_on_bottles = &effects::IgnoreArmorOnBottles{};

  let rbb = &effects::RerollBlackBlank{};
  let rab = &effects::RerollAnyBlank{};

  let hit_luck = &effects::LuckForHit{};
  let miss_luck = &effects::LuckForMiss{};
  let armor_luck = &effects::LuckForArmor{};
  let crit_luck = &effects::LuckForCrit{};

  println!(
    "cowboy: {:?}",
    attack::Disposition {
      dice: vec![
        dice::Red,
        dice::White,
        dice::Black,
        dice::Black,
      ],
      characteristics: attack::Characteristics {
        base_score: dice::new_scores(&[
          (dice::Damage, 1),
        ]),
        required_skill: 6,
        soft_armor: 2,
        hard_armor: 0,
      },
      effects: vec![],
      ..Default::default()
    }
    .average_scores()
  );

  println!(
    "sniper: {:?}",
    attack::Disposition {
      dice: vec![dice::Red, dice::White, dice::Green, dice::Green, dice::Blue],
      characteristics: attack::Characteristics {
        base_score: dice::new_scores(&[
          (dice::Damage, 2),
        ]),
        required_skill: 9,
        soft_armor: 5,
        hard_armor: 1,
      },

      effects: vec![shred_on_bottle, damage_on_star],
      ..Default::default()
    }
    .average_scores()
  );

  println!(
    "big guy: {:?}",
    attack::Disposition {
      dice: vec![
        dice::Red,
        dice::White,
        dice::Black,
        dice::Black,
        dice::Yellow,
        dice::Green,
        dice::Green
      ],
      characteristics: attack::Characteristics {
        base_score: dice::new_scores(&[
          (dice::Damage, 1),
        ]),
        required_skill: 9,
        soft_armor: 2,
        hard_armor: 1,
      },
      effects: vec![rbb, rab, hit_luck, miss_luck, ignore_armor_on_bottles, armor_luck, crit_luck],
      ..Default::default()
    }
    .average_scores()
  );

  println!(
    "result: {:?}",
    attack::Disposition {
      dice: vec![
        dice::Red,
        dice::White,
        dice::Black,
        dice::Black,
        dice::Yellow,
        dice::Green,
        dice::Green,
      ],
      characteristics: attack::Characteristics {
        base_score: dice::new_scores(&[
          (dice::Damage, 1),
        ]),
        required_skill: 7,
        soft_armor: 2,
        hard_armor: 1,
      },
      ..Default::default()
    }
    .average_scores()
  );
}
