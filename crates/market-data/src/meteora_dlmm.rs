use shared::types::{PoolConfig, PoolPrice};

pub fn decode_price_stub(pool: PoolConfig, slot: u64) -> PoolPrice {
    PoolPrice {
        pool,
        price: 0.0,
        slot,
    }
}
