use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, StdResult, Storage};
use cw_storage_plus::Item;

pub struct ContinuousWeightedAverage<'a>(Item<'a, State>);

#[cw_serde]
struct State {
    /// The total voting weight in the average.
    total: u64,
    /// The most recent average.
    average: Decimal,
}

impl<'a> ContinuousWeightedAverage<'a> {
    pub const fn new(key: &'a str) -> Self {
        Self(Item::new(key))
    }

    pub fn add(&self, storage: &mut dyn Storage, weight: u64, value: Decimal) -> StdResult<()> {
        if weight == 0 {
            return Ok(());
        }
        match self.0.may_load(storage)? {
            Some(State { total, average }) => {
                let total = total + weight;
                // average += weight * (value - average) / total
                let average = if value > average {
                    let diff = Decimal::from_atomics(weight, 0).unwrap() * (value - average)
                        / Decimal::from_atomics(total, 0).unwrap();
                    average + diff
                } else {
                    // multiply by (-1 * -1) and move the first -1
                    // into the (value - average) term to prevent
                    // overflow.
                    let diff = Decimal::from_atomics(weight, 0).unwrap() * (average - value)
                        / Decimal::from_atomics(total, 0).unwrap();
                    average - diff
                };
                self.0.save(storage, &State { total, average })
            }
            None => self.0.save(
                storage,
                &State {
                    total: weight,
                    average: value,
                },
            ),
        }
    }

    pub fn remove(&self, storage: &mut dyn Storage, weight: u64, value: Decimal) -> StdResult<()> {
        let State { total, average } = self
            .0
            .may_load(storage)?
            .expect("can not remove that which has not been added");
        // total * (average - weight * value / total) / (total - weight)
        if total == weight {
            self.0.remove(storage);
            Ok(())
        } else {
            let average = Decimal::from_atomics(total, 0).unwrap()
                * (average
                    - Decimal::from_atomics(weight, 0).unwrap() * value
                        / Decimal::from_atomics(total, 0).unwrap())
                / Decimal::from_atomics(total - weight, 0).unwrap();
            self.0.save(
                storage,
                &State {
                    total: total - weight,
                    average,
                },
            )
        }
    }

    pub fn average(&self, storage: &dyn Storage) -> StdResult<Decimal> {
        Ok(self
            .0
            .may_load(storage)?
            .unwrap_or(State {
                total: 0,
                average: Decimal::zero(),
            })
            .average)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;

    #[test]
    fn test_continuous_average() {
        let storage = &mut mock_dependencies().storage;
        let ave = ContinuousWeightedAverage::new("cwa");

        ave.add(storage, 4, Decimal::percent(1)).unwrap();
        let average = ave.average(storage).unwrap();
        assert_eq!(average, Decimal::percent(1));

        ave.add(storage, 3, Decimal::percent(10)).unwrap();
        let average = ave.average(storage).unwrap();
        assert!(average.to_string().starts_with("0.04857"));

        ave.add(storage, 2, Decimal::percent(8)).unwrap();
        let average = ave.average(storage).unwrap();
        assert!(average.to_string().starts_with("0.05555"));

        ave.remove(storage, 2, Decimal::percent(8)).unwrap();
        let average = ave.average(storage).unwrap();
        eprintln!("{}", average.to_string());
        assert!(average.to_string().starts_with("0.04857"));
    }
}
