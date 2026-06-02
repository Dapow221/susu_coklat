use std::collections::HashMap;

use shared::types::PoolPrice;

#[derive(Debug, Default)]
pub struct PriceCache {
    prices: HashMap<String, PoolPrice>,
}

impl PriceCache {
    pub fn upsert(&mut self, price: PoolPrice) {
        self.prices.insert(price.pool.address.clone(), price);
    }

    pub fn all(&self) -> Vec<PoolPrice> {
        self.prices.values().cloned().collect()
    }
}
