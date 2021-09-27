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
    collector: &mut attack::ResultCollector,
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
    collector.yield_last(case, 1.0)
  }
}

pub struct IgnoreArmorOnBottles {}

impl attack::Effect for IgnoreArmorOnBottles {
  fn yield_alterations<'a, 'b>(
    &self,
    collector: &mut attack::ResultCollector,
    case: &'a mut attack::Case<'b>,
  ) -> attack::LastAlteration {
    if case.result(attack::Hits) > 0.0 && case.score(dice::Bottle) > 0 {
      let scored_armor = if case.score(dice::Armor) > case.characteristics().soft_armor {
        0
      } else {
        case.score(dice::Armor)
      };
      let shredded_armor = case.score(dice::Shred).min(scored_armor);
      let aplied_soft_armor = scored_armor - shredded_armor;
      let scored_damage = case.score(dice::Damage);
      let full_armor = aplied_soft_armor + case.characteristics().hard_armor;
      let applied_armor = full_armor.min(scored_damage);
      let available_armor_negation = applied_armor.min(case.score(dice::Bottle));
      let bonus_score = full_armor - applied_armor + available_armor_negation;
      case.update(|updater| {
        *updater.score_mut(dice::Bottle) -= available_armor_negation;
        *updater.score_mut(dice::Damage) += bonus_score;
      });
      collector.yield_last(case, 1.0)
    } else {
      collector.yield_last(case, 1.0)
    }
  }
}

pub struct RerollBlackBlank {}

impl attack::Effect for RerollBlackBlank {
  fn yield_alterations(
    &self,
    collector: &mut attack::ResultCollector,
    case: &mut attack::Case,
  ) -> attack::LastAlteration {
    let blank_idx = case.roll().iter()
      .enumerate()
      .find(|(_, roll)| roll.die == dice::Black && roll.face == dice::Blank)
      .map(|needle| needle.0);
    if let Some(idx) = blank_idx {
      reroll(collector, case, dice::Black, idx)
    } else {
      collector.yield_last(case, 1.0)
    }
  }
}

pub struct RerollAnyBlank {}

impl attack::Effect for RerollAnyBlank {
  fn yield_alterations(
    &self,
    collector: &mut attack::ResultCollector,
    case: &mut attack::Case,
  ) -> attack::LastAlteration {
    let blank_idx = case.roll().iter()
      .cloned()
      .enumerate()
      .find(|(_, roll)| roll.die != dice::Green && roll.face == dice::Blank);
    if let Some((idx, roll)) = blank_idx {
      reroll(collector, case, roll.die, idx)
    } else if case.result(attack::Hits) > 0.0 && !near_hit(case, 2) {
      let blank_idx = case.roll().iter()
        .enumerate()
        .find(|(_, roll)| roll.die == dice::Green && roll.face == dice::Blank)
        .map(|needle| needle.0);
      if let Some(idx) = blank_idx {
        reroll(collector, case, dice::Green, idx)
      } else {
        collector.yield_last(case, 1.0)
      }
    } else if near_miss(case, 1) {
      let blank_idx = case.roll().iter()
        .enumerate()
        .find(|(_, roll)| roll.die == dice::Green && roll.face == dice::Blank)
        .map(|needle| needle.0);
      if let Some(idx) = blank_idx {
        reroll(collector, case, dice::Green, idx)
      } else {
        collector.yield_last(case, 1.0)
      }
    } else {
      collector.yield_last(case, 1.0)
    }
  }
}

// Luck effects should be added in an order as presented in this file. Otherwise it is possible to break the rules.
// E.g. use luck to generate a hit after luck was used to generate a miss.
pub struct LuckForHit {}

impl attack::Effect for LuckForHit {
  fn yield_alterations(
    &self,
    collector: &mut attack::ResultCollector,
    case: &mut attack::Case,
  ) -> attack::LastAlteration {
    if near_miss(case, 2) {
      collector.yield_next(case, 0.5);
      case.update(|updater| { *updater.score_mut(dice::Skill) -= 2 });
      collector.yield_last(case, 0.5)
    } else {
      collector.yield_last(case, 1.0)
    }
  }
}

pub struct LuckForMiss {}

impl attack::Effect for LuckForMiss {
  fn yield_alterations(
    &self,
    collector: &mut attack::ResultCollector,
    case: &mut attack::Case,
  ) -> attack::LastAlteration {
    if near_hit(case, 2) {
      collector.yield_next(case, 0.5);
      case.update(|updater| { *updater.score_mut(dice::Skill) += 2 });
      collector.yield_last(case, 0.5)
    } else {
      collector.yield_last(case, 1.0)
    }
  }
}

pub struct LuckForArmor {}

impl attack::Effect for LuckForArmor {
  fn yield_alterations(
    &self,
    collector: &mut attack::ResultCollector,
    case: &mut attack::Case,
  ) -> attack::LastAlteration {
    if case.result(attack::Hits) > 0.0 && case.result(attack::Damage) > 0.0 {
      collector.yield_next(case, 0.5);
      case.update(|updater| { *updater.score_mut(dice::Damage) -= 1 });
      collector.yield_last(case, 0.5)
    } else {
      collector.yield_last(case, 1.0)
    }
  }
}

pub struct LuckForCrit {}

impl attack::Effect for LuckForCrit {
  fn yield_alterations(
    &self,
    collector: &mut attack::ResultCollector,
    case: &mut attack::Case,
  ) -> attack::LastAlteration {
    if case.result(attack::Hits) > 0.0 {
      collector.yield_next(case, 0.5);
      case.update(|updater| { *updater.score_mut(dice::Crit) += 1 });
      collector.yield_last(case, 0.5)
    } else {
      collector.yield_last(case, 1.0)
    }
  }
}

fn reroll(collector: &mut attack::ResultCollector, case: &mut attack::Case, die: dice::Type, die_idx: usize) -> attack::LastAlteration {
  let faces = dice::RollFace::faces_of(die);
  for &face in faces.iter().skip(1) {
    case.update(|updater| { updater.roll_mut()[die_idx] = face; });
    collector.yield_next(case, face.probability);
  }
  case.update(|updater| { updater.roll_mut()[die_idx] = faces[0]; });
  collector.yield_last(case, faces[0].probability)
}

fn near_hit(case: &attack::Case, amount: Score) -> bool {
  case.result(attack::Hits) > 0.0 && case.score(dice::Hit) == 0 && case.score(dice::Skill) + amount > case.characteristics().required_skill
}

fn near_miss(case: &attack::Case, amount: Score) -> bool {
  case.result(attack::Hits) == 0.0 && case.score(dice::Miss) == 0 && case.score(dice::Skill) <= case.characteristics().required_skill + amount
}
