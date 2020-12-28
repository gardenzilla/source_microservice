use chrono::prelude::*;
use std::collections::HashMap;

// Price entity related to a SKU
pub struct PriceObject {
    net_price: u32,
    comment: String,
    created_at: DateTime<Utc>,
    created_by: String,
}

impl PriceObject
where
    Self: Sized,
{
    pub fn new(net_price: u32, comment: String, created_by: String) -> Self {
        Self {
            net_price,
            comment,
            created_at: Utc::now(),
            created_by,
        }
    }
}

// This represents a source as an entity
// Business data + SKU price list
pub struct Source {
    data: SourceData,
    prices: HashMap<u32, Vec<PriceObject>>,
    created_by: String,
    created_at: DateTime<Utc>,
}

// This represents a source business data
pub struct SourceData {
    id: u32,
    name: String,
    address: String,
    email: Vec<String>,
    phone: Vec<String>,
}
