use common::*;
use outcome;

outcome! {
    Real,
    Damage,
    Hits,
    Crits,
    Actions,
    BrokenLegs,
}

use dice;

#[derive(Default)]
pub struct Disposition<'a> {
    pub dice: Vec<dice::Die>,
    pub characteristics: Characteristics,
    pub effects: Vec<&'a dyn Effect>,
}

impl<'a> Disposition<'a> {
    pub fn average_scores(&self) -> self::Scores {
        dice::outcomes(&self.dice)
            .fold(Scores::default(), |mut acc, next| {
                let roll_result = self.attack_scores(next.scores);
                update(&mut acc, &roll_result, |sum, part| {
                    *sum += *part * next.probability;
                });
                acc
            })
    }

    fn attack_scores(&self, roll: dice::Scores) -> Scores {
        let input = Input { characteristics: self.characteristics.clone(), roll: roll };
        ResultCollector::new(input, self.effects.clone()).collect().scores
    }
}

#[derive(Clone, Default)]
pub struct Characteristics {
    pub base_score: dice::Scores,
    pub required_skill: Score,
    pub soft_armor: Score,
    pub hard_armor: Score,
}

#[derive(Clone)]
pub struct Input {
    pub characteristics: Characteristics,
    pub roll: dice::Scores,
}

impl Input {
    pub fn compute_output(&self) -> Output {
        let mut roll = self.roll.clone();

        dice::update(&mut roll, &self.characteristics.base_score, |sum, part| {
            *sum += *part;
        });

        let miss = roll[dice::Miss] > 0 || roll[dice::Skill] > self.characteristics.required_skill;
        let hit = roll[dice::Hit] > 0 || !miss;
        let scored_armor = if roll[dice::Armor] > self.characteristics.soft_armor {
            0
        } else {
            roll[dice::Armor]
        };
        let shredded_armor = roll[dice::Shred].min(scored_armor);
        let aplied_soft_armor = scored_armor - shredded_armor;
        let damage = roll[dice::Damage];
        let applied_armor = (aplied_soft_armor + self.characteristics.hard_armor).min(damage);
        let damage = damage - applied_armor;
        let mut scores = Scores::default();
        scores[Damage] = if hit { damage } else { 0 } as Real;
        scores[Hits] = if hit { 1.0 } else { 0.0 };
        scores[Crits] = roll[dice::Crit] as Real;
        scores[Actions] = roll[dice::Action] as Real;
        Output { scores }
    }
}

#[derive(Clone, Default)]
pub struct Output {
    pub scores: Scores,
}

impl Output {
    fn encompass(&mut self, other: &Output, prob: Real) {
        update(&mut self.scores, &other.scores, |sum, part| {
            *sum += *part * prob;
        });
    }
}

#[derive(Clone)]
pub struct Occurence {
    input: Input,
    output: Output,
}

impl Occurence {
    pub fn update(&mut self, f: impl Fn(&mut OccurenceUpdater)) {
        f(&mut OccurenceUpdater { occurence: self });
        self.output = self.input.compute_output();
    }

    pub fn result(&mut self, facet: Facet) -> Real {
        self.output.scores[facet]
    }

    pub fn score(&self, facet: dice::Facet) -> Score {
        self.input.roll[facet] + self.input.characteristics.base_score[facet]
    }
}

pub struct OccurenceUpdater<'a> {
    occurence: &'a mut Occurence,
}

impl<'a> OccurenceUpdater<'a> {
    pub fn score_mut(&mut self, facet: dice::Facet) -> &mut Score {
        &mut self.occurence.input.characteristics.base_score[facet]
    }
}

pub struct ResultCollector<'a> {
    effects: Vec<&'a dyn Effect>,
    effect_idx: usize,
    input: Option<Input>,
    output: Output,
    probs: Vec<Real>,
}

impl<'a> ResultCollector<'a> {
    pub fn yield_(&mut self, occurence: &mut Occurence, prob: Real) {
        self.probs[self.effect_idx] = prob;
        self.effect_idx += 1;
        if self.effect_idx == self.effects.len() {
            self.output.encompass(&occurence.output, self.probs.iter().product())
        } else {
            self.effects[self.effect_idx].yield_alterations(self, occurence)
        }
    }

    fn collect(mut self) -> Output {
        let input = self.input.take().unwrap();
        let output = input.compute_output();
        let mut occurence = Occurence { input, output };
        if self.effects.len() > 0 {
            self.effects[self.effect_idx].yield_alterations(&mut self, &mut occurence);
        } else {
            self.output = occurence.output;
        }
        self.output
    }

    fn new(input: Input, effects: Vec<&'a dyn Effect>) -> ResultCollector {
        let len = effects.len();
        ResultCollector {
            effects: effects,
            effect_idx: 0,
            input: Some(input),
            output: Default::default(),
            probs: vec![1.0; len]
        }
    }
}

pub trait Effect {
    fn yield_alterations(&self, collector: &mut ResultCollector, occurence: &mut Occurence);
}
