use common::*;
use outcome;

outcome! {
  Score,
  Damage,
  Shred,
  Skill,
  Bottle,
  Star,
  Explosion,
  Armor,
  Crit,
  Action,
  Hit,
  Miss,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Type {
  White,
  Red,
  Green,
  Yellow,
  Black,
  Blue,
}

pub use self::Type::*;

#[derive(Copy, Clone, PartialEq)]
pub enum Face {
  Skill2,
  Skill3,
  Skill4,
  Skill5,
  Skill6,
  Skill7,
  Skill8,
  Skill9,
  Skill10,
  SkillMinus1,
  SkillMinus2,
  SkillMinus3,
  Blank,
  Damage1,
  Damage2,
  Shred1,
  Shred2,
  Bottle1,
  Bottle2,
  Star1,
  Star2,
  BottleStar,
  Explosion1,
  Crit1,
  Action1,
  Miss1,
  MissAction,
  Armor1,
  Armor2,
  Armor3,
  Armor4,
}

pub use self::Face::*;

#[derive(Clone, Copy)]
pub struct RollFace {
  pub die: Type,
  pub face: Face,
  pub probability: Real,
}

impl RollFace {

  pub fn add_score(&self, outcome: &mut ::dice::Outcome) {
    outcome.probability *= self.probability;
    match self.face {
      Skill2 => outcome.scores[Facet::Skill] += 2,
      Skill3 => outcome.scores[Facet::Skill] += 3,
      Skill4 => outcome.scores[Facet::Skill] += 4,
      Skill5 => outcome.scores[Facet::Skill] += 5,
      Skill6 => outcome.scores[Facet::Skill] += 6,
      Skill7 => outcome.scores[Facet::Skill] += 7,
      Skill8 => outcome.scores[Facet::Skill] += 8,
      Skill9 => outcome.scores[Facet::Skill] += 9,
      Skill10 => outcome.scores[Facet::Skill] += 10,
      SkillMinus1 => outcome.scores[Facet::Skill] -= 1,
      SkillMinus2 => outcome.scores[Facet::Skill] -= 2,
      SkillMinus3 => outcome.scores[Facet::Skill] -= 3,
      Blank => match self.die {
        Green => outcome.scores[Facet::Skill] -= 2,
        _ => (),
      },
      Damage1 => outcome.scores[Facet::Damage] += 1,
      Damage2 => outcome.scores[Facet::Damage] += 2,
      Shred1 => outcome.scores[Facet::Shred] += 1,
      Shred2 => outcome.scores[Facet::Shred] += 2,
      Bottle1 => outcome.scores[Facet::Bottle] += 1,
      Bottle2 => outcome.scores[Facet::Bottle] += 2,
      Star1 => outcome.scores[Facet::Star] += 1,
      Star2 => outcome.scores[Facet::Star] += 2,
      BottleStar => {
        outcome.scores[Facet::Bottle] += 1;
        outcome.scores[Facet::Star] += 1;
      },
      Explosion1 => outcome.scores[Facet::Explosion] += 1,
      Crit1 => {
        outcome.scores[Facet::Crit] += 1;
        outcome.scores[Facet::Hit] += 1;
      }
      Action1 => {
        outcome.scores[Facet::Action] += 1;
        outcome.scores[Facet::Skill] += 1;
      }
      Miss1 => outcome.scores[Facet::Miss] = 1,
      MissAction => {
        outcome.scores[Facet::Action] += 1;
        outcome.scores[Facet::Miss] = 1;
      }
      Armor1 => outcome.scores[Facet::Armor] += 1,
      Armor2 => outcome.scores[Facet::Armor] += 2,
      Armor3 => outcome.scores[Facet::Armor] += 3,
      Armor4 => outcome.scores[Facet::Armor] += 4,
    }
  }

  pub fn faces_of(die: Type) -> &'static [RollFace] {
    static d12: Real = 1.0 / 12.0;
    static d20: Real = 1.0 / 20.0;
    match die {
      White => {
        static ret: &'static [RollFace] = &[
          RollFace { die: White, face: Action1, probability: 2.0 * d20 },
          RollFace { die: White, face: Crit1, probability: 3.0 * d20 },
          RollFace { die: White, face: Skill2, probability: 1.0 * d20 },
          RollFace { die: White, face: Skill3, probability: 1.0 * d20 },
          RollFace { die: White, face: Skill4, probability: 1.0 * d20 },
          RollFace { die: White, face: Skill5, probability: 1.0 * d20 },
          RollFace { die: White, face: Skill6, probability: 2.0 * d20 },
          RollFace { die: White, face: Skill7, probability: 2.0 * d20 },
          RollFace { die: White, face: Skill8, probability: 2.0 * d20 },
          RollFace { die: White, face: Skill9, probability: 1.0 * d20 },
          RollFace { die: White, face: Skill10, probability: 1.0 * d20 },
          RollFace { die: White, face: Miss1, probability: 2.0 * d20 },
          RollFace { die: White, face: MissAction, probability: 1.0 * d20 },
        ];
        ret
      },
      Red => {
        static ret: &'static [RollFace] = &[
          RollFace { die: Red, face: Armor1, probability: 4.0 * d12 },
          RollFace { die: Red, face: Armor2, probability: 3.0 * d12 },
          RollFace { die: Red, face: Armor3, probability: 3.0 * d12 },
          RollFace { die: Red, face: Armor4, probability: 2.0 * d12 },
        ];
        ret
      },
      Black => {
        static ret: &'static [RollFace] = &[
          RollFace { die: Black, face: Damage1, probability: 5.0 * d12 },
          RollFace { die: Black, face: Damage2, probability: 1.0 * d12 },
          RollFace { die: Black, face: Blank, probability: 3.0 * d12 },
          RollFace { die: Black, face: Bottle1, probability: 1.0 * d12 },
          RollFace { die: Black, face: Shred1, probability: 1.0 * d12 },
          RollFace { die: Black, face: SkillMinus1, probability: 1.0 * d12 },
        ];
        ret
      },
      Green => {
        static ret: &'static [RollFace] = &[
          RollFace { die: Green, face: Blank, probability: 4.0 * d12 },
          RollFace { die: Green, face: SkillMinus2, probability: 2.0 * d12 },
          RollFace { die: Green, face: SkillMinus3, probability: 1.0 * d12 },
          RollFace { die: Green, face: Shred1, probability: 2.0 * d12 },
          RollFace { die: Green, face: Damage1, probability: 2.0 * d12 },
          RollFace { die: Green, face: Bottle1, probability: 1.0 * d12 },
        ];
        ret
      },
      Yellow => {
        static ret: &'static [RollFace] = &[
          RollFace { die: Yellow, face: Shred1, probability: 5.0 * d12 },
          RollFace { die: Yellow, face: Shred2, probability: 1.0 * d12 },
          RollFace { die: Yellow, face: Blank, probability: 2.0 * d12 },
          RollFace { die: Yellow, face: Bottle1, probability: 1.0 * d12 },
          RollFace { die: Yellow, face: SkillMinus1, probability: 2.0 * d12 },
          RollFace { die: Yellow, face: Damage1, probability: 1.0 * d12 },
        ];
        ret
      },
      Blue => {
        static ret: &'static [RollFace] = &[
          RollFace { die: Blue, face: Bottle1, probability: 4.0 * d12 },
          RollFace { die: Blue, face: Bottle2, probability: 2.0 * d12 },
          RollFace { die: Blue, face: BottleStar, probability: 1.0 * d12 },
          RollFace { die: Blue, face: Star1, probability: 2.0 * d12 },
          RollFace { die: Blue, face: Star2, probability: 1.0 * d12 },
          RollFace { die: Blue, face: Explosion1, probability: 2.0 * d12 },
        ];
        ret
      }
    }
  }
}
