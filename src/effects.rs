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
      let faces = dice::RollFace::faces_of(dice::Black);
      for &face in faces.iter().skip(1) {
        case.update(|updater| { updater.roll_mut()[idx] = face; });
        collector.yield_next(case, face.probability);
      }
      case.update(|updater| { updater.roll_mut()[idx] = faces[0]; });
      collector.yield_last(case, faces[0].probability)
    } else {
      collector.yield_last(case, 1.0)
    }
  }
}

// yield last should take ownership of occurence - so we cannot call it multiple times (we are not required to return). And we can remove stupid return thing then.
// yield next will create new occurence with borrowed cows, so future effects won't mutate values current effect is still going to use.
