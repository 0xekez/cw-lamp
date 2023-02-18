use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::{Item, Map};

use crate::continuous_average::ContinuousWeightedAverage;

pub(crate) const CW4: Item<Addr> = Item::new("cw4");
pub(crate) const PREFERENCES: Map<&Addr, Decimal> = Map::new("p");
pub(crate) const CWA: ContinuousWeightedAverage = ContinuousWeightedAverage::new("cwa");
