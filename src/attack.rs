use std::marker::PhantomData;
use std::borrow::{ Cow, Borrow };
use common::*;
use cartesian_fold::IterExt;
use outcome;
use dice;

outcome! {
  Real,
  Damage,
  Hits,
  Crits,
  Actions,
  BrokenLegs,
}

#[derive(Default)]
pub struct Disposition<'a> {
  pub dice: Vec<dice::Type>,
  pub characteristics: Characteristics,
  pub effects: Vec<&'a dyn Effect>,
}

impl<'a> Disposition<'a> {
  pub fn average_scores(&self) -> self::Scores {
    self.dice
      .iter()
      .map(|&die| { dice::RollFace::faces_of(die).iter().cloned() })
      .cartesian_collections(|faces| { self.attack_scores(faces) })
      .fold(Scores::default(), |mut acc, next| {
        update(&mut acc, &next.scores, |sum, part| {
          *sum += *part * next.probability;
        });
        acc
      })
  }

  fn attack_scores(&self, roll: &[dice::RollFace]) -> Outcome {
    EffectApplicator::new(self.effects.clone())
      .apply_effects(&self.characteristics, roll)
  }
}

#[derive(Clone, Default)]
pub struct Characteristics {
  pub base_score: dice::Scores,
  pub required_skill: Score,
  pub soft_armor: Score,
  pub hard_armor: Score,
}

pub fn compute_output(characteristics: &Characteristics, roll: &[dice::RollFace]) -> (dice::Outcome, Scores) {
  let mut roll_outcome = roll.iter().fold(dice::Outcome::new(), |mut acc, face| {
    face.add_score(&mut acc);
    acc
  });
  let roll = &mut roll_outcome.scores;

  dice::update(roll, &characteristics.base_score, |sum, part| {
    *sum += *part;
  });

  let miss = roll[dice::Miss] > 0 || roll[dice::Skill] > characteristics.required_skill;
  let hit = roll[dice::Hit] > 0 || !miss;
  let scored_armor = roll[dice::Armor];
  let shredded_armor = (characteristics.soft_armor - roll[dice::Shred]).max(0);
  let aplied_soft_armor = if scored_armor > shredded_armor { 0 } else { scored_armor };
  let damage = roll[dice::Damage];
  let applied_armor = (aplied_soft_armor + characteristics.hard_armor).min(damage);
  let damage = damage - applied_armor;
  let mut scores = Scores::default();
  scores[Damage] = if hit { damage } else { 0 } as Real;
  scores[Hits] = if hit { 1.0 } else { 0.0 };
  scores[Crits] = roll[dice::Crit] as Real;
  scores[Actions] = roll[dice::Action] as Real;
  (roll_outcome, scores)
}

pub struct Case<'a> {
  characteristics: Cow<'a, Characteristics>,
  dice_roll: Cow<'a, [dice::RollFace]>,
  dice_outcome: dice::Outcome,
  scores: Scores,
}

impl<'a> Case<'a> {
  pub fn update(&mut self, f: impl Fn(&mut CaseUpdater::<'_, 'a>)) {
    f(&mut CaseUpdater { case: self });
    let (dice_outcome, scores) = compute_output(&self.characteristics, &self.dice_roll);
    self.dice_outcome = dice_outcome;
    self.scores = scores;
  }

  pub fn result(&self, facet: Facet) -> Real {
    self.scores[facet]
  }

  pub fn score(&self, facet: dice::Facet) -> Score {
    self.dice_outcome.scores[facet]
  }

  pub fn characteristics(&self) -> &Characteristics {
    &self.characteristics
  }

  pub fn roll(&self) -> &[dice::RollFace] {
    &self.dice_roll
  }

  pub fn reborrow(&'a self) -> Self {
    Case {
      characteristics: Cow::Borrowed(self.characteristics.borrow()),
      dice_roll: Cow::Borrowed(&self.dice_roll),
      dice_outcome: self.dice_outcome.clone(),
      scores: self.scores,
    }
  }
}

pub struct CaseUpdater<'u, 'o> {
  case: &'u mut Case<'o>,
}

impl<'u, 'o> CaseUpdater<'u, 'o> {
  pub fn score_mut(&mut self, facet: dice::Facet) -> &mut Score {
    &mut self.case.characteristics.to_mut().base_score[facet]
  }

  pub fn roll_mut(&mut self) -> &mut [dice::RollFace] {
    self.case.dice_roll.to_mut()
  }
}

pub struct EffectApplicator<'a> {
  effects: Vec<&'a dyn Effect>,
  effect_idx: usize,
  outcome: Outcome,
  probs: Vec<Real>,
}

pub struct LastAlteration { opaque: PhantomData<Opaque> }
struct Opaque;

impl<'a> EffectApplicator<'a> {
  pub fn yield_next_alteration(&mut self, case: &mut Case, prob: Real) {
    self.probs[self.effect_idx] = prob;
    if self.effect_idx == self.effects.len() - 1 {
      self.update_after_last_effect(case);
    } else {
      let mut case = case.reborrow();
      self.effect_idx += 1;
      self.effects[self.effect_idx].yield_alterations(self, &mut case);
      self.effect_idx -= 1;
    }
  }

  pub fn yield_last_alteration(&mut self, case: &mut Case, prob: Real) -> LastAlteration {
    self.probs[self.effect_idx] = prob;
    if self.effect_idx == self.effects.len() - 1 {
      self.update_after_last_effect(case);
    } else {
      self.effect_idx += 1;
      self.effects[self.effect_idx].yield_alterations(self, case);
      self.effect_idx -= 1;
    }
    LastAlteration { opaque: PhantomData }
  }

  fn update_after_last_effect(&mut self, case: &Case) {
    let prob: Real = self.probs.iter().product();
    update(&mut self.outcome.scores, &case.scores, |sum, part| {
      *sum += *part * prob;
    });
  }

  fn apply_effects(mut self, characteristics: &Characteristics, roll: &[dice::RollFace]) -> Outcome {
    let (dice_outcome, scores) = compute_output(characteristics, roll);
    self.outcome.probability = dice_outcome.probability;
    if self.effects.len() > 0 {
      let mut case = Case {
        characteristics: Cow::Borrowed(characteristics),
        dice_roll: Cow::Borrowed(roll),
        dice_outcome,
        scores
      };
      self.effects[self.effect_idx].yield_alterations(&mut self, &mut case);
    } else {
      self.outcome.scores = scores;
    }
    self.outcome
  }

  fn new(effects: Vec<&'a dyn Effect>) -> EffectApplicator<'a> {
    let len = effects.len();
    EffectApplicator {
      effects,
      effect_idx: 0,
      outcome: Default::default(),
      probs: vec![1.0; len],
    }
  }
}

pub trait Effect {
  fn yield_alterations<'a, 'b>(&self, collector: &mut EffectApplicator, case: &'a mut Case<'b>) -> LastAlteration;
}

#[cfg(test)]
mod test {
  use attack::*;

  fn roll(face: dice::Face) -> dice::RollFace {
    dice::RollFace { die: dice::White, face, probability: 0.0 }
  }

  #[test]
  fn roll_exact_hits() {
    let chars = &Characteristics {
      required_skill: 6,
      ..Default::default()
    };
    let output = compute_output(chars, &[roll(dice::Skill6)]);
    assert_eq!(output.1[Hits].round() as i64, 1);
  }

  #[test]
  fn high_roll_misses() {
    let chars = &Characteristics {
      required_skill: 6,
      ..Default::default()
    };
    let output = compute_output(chars, &[roll(dice::Skill7)]);
    assert_eq!(output.1[Hits].round() as i64, 0);
  }

  #[test]
  fn probability_multiplies() {
    fn prob(prob: Real) -> dice::RollFace {
      dice::RollFace { die: dice::White, face: dice::Crit1, probability: prob}
    }
    let output = compute_output(&Default::default(), &[prob(2.0), prob(5.0), prob(1.5)]);
    assert_eq!(output.0.probability.round() as i64, 15);
  }

  #[test]
  fn hard_armor_applies() {
    let chars = &Characteristics {
      base_score: dice::new_scores(&[(dice::Damage, 5), (dice::Hit, 1)]),
      hard_armor: 1,
      ..Default::default()
    };
    let output = compute_output(&chars, &[]);
    assert_eq!(output.1[Damage].round() as i64, 4);
  }

  #[test]
  fn soft_armor_low_roll() {
    let chars = &Characteristics {
      base_score: dice::new_scores(&[(dice::Damage, 5), (dice::Hit, 1)]),
      soft_armor: 3,
      ..Default::default()
    };
    let output = compute_output(&chars, &[roll(dice::Armor2)]);
    assert_eq!(output.1[Damage].round() as i64, 3);
  }

  #[test]
  fn soft_armor_high_roll() {
    let chars = &Characteristics {
      base_score: dice::new_scores(&[(dice::Damage, 5), (dice::Hit, 1)]),
      soft_armor: 3,
      ..Default::default()
    };
    let output = compute_output(&chars, &[roll(dice::Armor4)]);
    assert_eq!(output.1[Damage].round() as i64, 5);
  }

  #[test]
  fn damage_is_summed() {
    let chars = &Characteristics {
      base_score: dice::new_scores(&[(dice::Damage, 5), (dice::Hit, 1)]),
      ..Default::default()
    };
    let output = compute_output(&chars, &[roll(dice::Damage1), roll(dice::Damage2)]);
    assert_eq!(output.1[Damage].round() as i64, 8);
  }

  #[test]
  fn shred_ignores_armor() {
    let chars = &Characteristics {
      base_score: dice::new_scores(&[(dice::Damage, 5), (dice::Hit, 1)]),
      soft_armor: 3,
      ..Default::default()
    };
    let output = compute_output(&chars, &[roll(dice::Armor3), roll(dice::Shred1)]);
    assert_eq!(output.1[Damage].round() as i64, 5);
  }

  #[test]
  fn damage_on_hits_only() {
    let disposition = Disposition {
      characteristics: Characteristics {
        required_skill: 9,
        base_score: dice::new_scores(&[(dice::Damage, 1)]),
        ..Default::default()
      },
      dice: vec![dice::White],
      ..Default::default()
    };
    let outcome = disposition.average_scores();
    assert_eq!((outcome[Damage] * 20.0).round() as i64,  16);
  }

  #[test]
  fn soft_armor_statistics() {
    let disposition = Disposition {
      characteristics: Characteristics {
        base_score: dice::new_scores(&[(dice::Damage, 3), (dice::Hit, 1)]),
        soft_armor: 2,
        ..Default::default()
      },
      dice: vec![dice::Red],
      ..Default::default()
    };
    let outcome = disposition.average_scores();
    assert_eq!((outcome[Damage] * 12.0).round() as i64,  26);
  }

  #[test]
  fn black_2_dice_statistics() {
    let disposition = Disposition {
      dice: vec![dice::Black, dice::Black, dice::Blue],
      characteristics: Characteristics {
        base_score: dice::new_scores(&[(dice::Hit, 1)]),
        ..Default::default()
      },
      ..Default::default()
    };
    let outcome = disposition.average_scores();
    assert_eq!((outcome[Damage] * 144.0).round() as i64 ,  168);
  }
}