use attack;
use dice;
use common::*;

pub struct ExchangeScoreOnHit {
  pub give: (dice::Facet, PositiveScore),
  pub take: (dice::Facet, PositiveScore),
  pub times: Option<PositiveScore>,
}

impl attack::Effect for ExchangeScoreOnHit {
  fn yield_alterations<'a, 'b>(
    &self,
    applicator: &mut attack::EffectApplicator,
    case: &'a mut attack::Case<'b>,
  ) -> attack::LastAlteration {
    if case.result(attack::Hits) > 0.0 {
      let available = case.score(self.give.0);
      let times = available / self.give.1.get();
      let times = match self.times {
        Some(non_zero) => times.min(non_zero.get()),
        None => times,
      };
      if times > 0 {
        case.update(|updater| {
          *updater.score_mut(self.give.0) -= times * self.give.1.get();
          *updater.score_mut(self.take.0) += times * self.take.1.get();
        });
      }
    }
    applicator.yield_last_alteration(case, 1.0)
  }
}

pub struct IgnoreArmorOnBottles {}

impl attack::Effect for IgnoreArmorOnBottles {
  fn yield_alterations<'a, 'b>(
    &self,
    applicator: &mut attack::EffectApplicator,
    case: &'a mut attack::Case<'b>,
  ) -> attack::LastAlteration {
    if case.result(attack::Hits) > 0.0 && case.score(dice::Bottle) > 0 {
      let scored_armor = case.score(dice::Armor);
      let shredded_armor = case.characteristics().soft_armor - case.score(dice::Shred);
      let activated_armor = if scored_armor > shredded_armor { 0 } else { scored_armor };
      let scored_damage = case.score(dice::Damage);
      let available_armor = activated_armor + case.characteristics().hard_armor;
      let potential_effect_damage = available_armor.min(scored_damage).min(case.score(dice::Bottle));
      let current_result_damage = scored_damage - available_armor;
      if potential_effect_damage > current_result_damage {
        let bonus_score = potential_effect_damage - current_result_damage;
        case.update(|updater| {
          *updater.score_mut(dice::Bottle) -= potential_effect_damage;
          *updater.score_mut(dice::Damage) += bonus_score;
        });
      }
    }
    applicator.yield_last_alteration(case, 1.0)
  }
}

pub struct RerollBlackBlank {}

impl attack::Effect for RerollBlackBlank {
  fn yield_alterations(
    &self,
    applicator: &mut attack::EffectApplicator,
    case: &mut attack::Case,
  ) -> attack::LastAlteration {
    let blank_idx = case.roll().iter()
      .enumerate()
      .find(|(_, roll)| roll.die == dice::Black && roll.face == dice::Blank)
      .map(|needle| needle.0);
    if let Some(idx) = blank_idx {
      reroll(applicator, case, dice::Black, idx)
    } else {
      applicator.yield_last_alteration(case, 1.0)
    }
  }
}

pub struct RerollAnyBlank {}

impl attack::Effect for RerollAnyBlank {
  fn yield_alterations(
    &self,
    applicator: &mut attack::EffectApplicator,
    case: &mut attack::Case,
  ) -> attack::LastAlteration {
    let blank_idx = case.roll().iter()
      .cloned()
      .enumerate()
      .find(|(_, roll)| roll.die != dice::Green && roll.face == dice::Blank);
    if let Some((idx, roll)) = blank_idx {
      reroll(applicator, case, roll.die, idx)
    } else if case.result(attack::Hits) > 0.0 && !near_hit(case, 2) {
      let blank_idx = case.roll().iter()
        .enumerate()
        .find(|(_, roll)| roll.die == dice::Green && roll.face == dice::Blank)
        .map(|needle| needle.0);
      if let Some(idx) = blank_idx {
        reroll(applicator, case, dice::Green, idx)
      } else {
        applicator.yield_last_alteration(case, 1.0)
      }
    } else if near_miss(case, 1) {
      let blank_idx = case.roll().iter()
        .enumerate()
        .find(|(_, roll)| roll.die == dice::Green && roll.face == dice::Blank)
        .map(|needle| needle.0);
      if let Some(idx) = blank_idx {
        reroll(applicator, case, dice::Green, idx)
      } else {
        applicator.yield_last_alteration(case, 1.0)
      }
    } else {
      applicator.yield_last_alteration(case, 1.0)
    }
  }
}

// Luck effects should be added in an order as presented in this file. Otherwise it is possible to break the rules.
// E.g. use luck to generate a hit after luck was used to generate a miss.
pub struct LuckForHit {}

impl attack::Effect for LuckForHit {
  fn yield_alterations(
    &self,
    applicator: &mut attack::EffectApplicator,
    case: &mut attack::Case,
  ) -> attack::LastAlteration {
    if near_miss(case, 2) {
      applicator.yield_next_alteration(case, 0.5);
      case.update(|updater| { *updater.score_mut(dice::Skill) -= 2 });
      applicator.yield_last_alteration(case, 0.5)
    } else {
      applicator.yield_last_alteration(case, 1.0)
    }
  }
}

pub struct LuckForMiss {}

impl attack::Effect for LuckForMiss {
  fn yield_alterations(
    &self,
    applicator: &mut attack::EffectApplicator,
    case: &mut attack::Case,
  ) -> attack::LastAlteration {
    if near_hit(case, 2) {
      applicator.yield_next_alteration(case, 0.5);
      case.update(|updater| { *updater.score_mut(dice::Skill) += 2 });
      applicator.yield_last_alteration(case, 0.5)
    } else {
      applicator.yield_last_alteration(case, 1.0)
    }
  }
}

pub struct LuckForArmor {}

impl attack::Effect for LuckForArmor {
  fn yield_alterations(
    &self,
    applicator: &mut attack::EffectApplicator,
    case: &mut attack::Case,
  ) -> attack::LastAlteration {
    if case.result(attack::Hits) > 0.0 && case.result(attack::Damage) > 0.0 {
      applicator.yield_next_alteration(case, 0.5);
      case.update(|updater| { *updater.score_mut(dice::Damage) -= 1 });
      applicator.yield_last_alteration(case, 0.5)
    } else {
      applicator.yield_last_alteration(case, 1.0)
    }
  }
}

pub struct LuckForCrit {}

impl attack::Effect for LuckForCrit {
  fn yield_alterations(
    &self,
    applicator: &mut attack::EffectApplicator,
    case: &mut attack::Case,
  ) -> attack::LastAlteration {
    if case.result(attack::Hits) > 0.0 {
      applicator.yield_next_alteration(case, 0.5);
      case.update(|updater| { *updater.score_mut(dice::Crit) += 1 });
      applicator.yield_last_alteration(case, 0.5)
    } else {
      applicator.yield_last_alteration(case, 1.0)
    }
  }
}

fn reroll(applicator: &mut attack::EffectApplicator, case: &mut attack::Case, die: dice::Type, die_idx: usize) -> attack::LastAlteration {
  let faces = dice::RollFace::faces_of(die);
  for &face in faces.iter().skip(1) {
    case.update(|updater| { updater.roll_mut()[die_idx] = face; });
    applicator.yield_next_alteration(case, face.probability);
  }
  case.update(|updater| { updater.roll_mut()[die_idx] = faces[0]; });
  applicator.yield_last_alteration(case, faces[0].probability)
}

fn near_hit(case: &attack::Case, amount: Score) -> bool {
  case.result(attack::Hits) > 0.0 && case.score(dice::Hit) == 0 && case.score(dice::Skill) + amount > case.characteristics().required_skill
}

fn near_miss(case: &attack::Case, amount: Score) -> bool {
  case.result(attack::Hits) == 0.0 && case.score(dice::Miss) == 0 && case.score(dice::Skill) <= case.characteristics().required_skill + amount
}

#[cfg(test)]
mod test {
  use effects::*;
  use attack;

  #[test]
  fn ignore_one_armor_on_bottle() {
    let effect = &IgnoreArmorOnBottles{};
    let disposition = attack::Disposition {
      dice: vec![dice::Red, dice::Blue],
      characteristics: attack::Characteristics {
        base_score: dice::new_scores(&[(dice::Hit, 1), (dice::Damage, 1)]),
        soft_armor: 1,
        ..Default::default()
      },
      effects: vec![effect],
      ..Default::default()
    };
    let outcome = disposition.average_scores();

    assert_eq!((outcome[attack::Damage] * 144.0).round() as i64, 124);
  }
}