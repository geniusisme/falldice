use attack;
use dice;
use common::*;

pub struct ExchangeScoreOnHit {
  pub give: (dice::Facet, PositiveScore),
  pub take: (dice::Facet, PositiveScore),
  pub times: Option<PositiveScore>,
}

impl attack::Effect for ExchangeScoreOnHit {
  fn yield_alterations(
    &self,
    collector: &mut attack::ResultCollector,
    occurence: &mut attack::Occurence,
  ) {
    if occurence.result(attack::Hits) > 0.0 {
      let available = occurence.score(self.give.0);
      let times = available / self.give.1.get();
      let times = match self.times {
        Some(non_zero) => times.min(non_zero.get()),
        None => times,
      };
      if times > 0 {
        occurence.update(|updater| {
          *updater.score_mut(self.give.0) -= times * self.give.1.get();
          *updater.score_mut(self.take.0) += times * self.take.1.get();
        });
        collector.yield_(occurence, 1.0);
      } else {
        collector.yield_(occurence, 1.0)
      }
    } else {
      collector.yield_(occurence, 1.0)
    }
  }
}

// todo: replace input+output with one quite opaque struct;
// one can query dice scores to get sum result and to take away from them
// but still we want to be able to change charateristics
// output is automatically created with this struct
