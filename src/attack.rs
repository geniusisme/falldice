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
    ResultCollector::new(self.effects.clone())
      .collect(&self.characteristics, roll)
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
  let scored_armor = if roll[dice::Armor] > characteristics.soft_armor {
    0
  } else {
    roll[dice::Armor]
  };
  let shredded_armor = roll[dice::Shred].min(scored_armor);
  let aplied_soft_armor = scored_armor - shredded_armor;
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

  pub fn result(&mut self, facet: Facet) -> Real {
    self.scores[facet]
  }

  pub fn score(&self, facet: dice::Facet) -> Score {
    self.dice_outcome.scores[facet] + self.characteristics.base_score[facet]
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

pub struct ResultCollector<'a> {
  effects: Vec<&'a dyn Effect>,
  effect_idx: usize,
  outcome: Outcome,
  probs: Vec<Real>,
}

pub struct LastAlteration { opaque: PhantomData<Opaque> }
struct Opaque;

impl<'a> ResultCollector<'a> {
  pub fn yield_next(&mut self, case: &mut Case, prob: Real) {
    self.probs[self.effect_idx] = prob;
    if self.effect_idx == self.effects.len() - 1 {
      self.collect_after_last_effect(case);
    } else {
      let mut case = case.reborrow();
      self.effect_idx += 1;
      self.effects[self.effect_idx].yield_alterations(self, &mut case);
      self.effect_idx -= 1;
    }
  }

  pub fn yield_last(&mut self, case: &mut Case, prob: Real) -> LastAlteration {
    self.probs[self.effect_idx] = prob;
    if self.effect_idx == self.effects.len() - 1 {
      self.collect_after_last_effect(case);
    } else {
      self.effect_idx += 1;
      self.effects[self.effect_idx].yield_alterations(self, case);
      self.effect_idx -= 1;
    }
    LastAlteration { opaque: PhantomData }
  }

  fn collect_after_last_effect(&mut self, case: &Case) {
    let prob: Real = self.probs.iter().product();
    update(&mut self.outcome.scores, &case.scores, |sum, part| {
      *sum += *part * prob;
    });
  }

  fn collect(mut self, characteristics: &Characteristics, roll: &[dice::RollFace]) -> Outcome {
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

  fn new(effects: Vec<&'a dyn Effect>) -> ResultCollector<'a> {
    let len = effects.len();
    ResultCollector {
      effects,
      effect_idx: 0,
      outcome: Default::default(),
      probs: vec![1.0; len],
    }
  }
}

pub trait Effect {
  fn yield_alterations<'a, 'b>(&self, collector: &mut ResultCollector, case: &'a mut Case<'b>) -> LastAlteration;
}
