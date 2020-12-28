use chrono::prelude::*;
use std::collections::HashMap;

// Price entity related to a SKU
#[derive(Debug, Clone)]
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
  id: u32,
  data: SourceData,
  prices: HashMap<u32, Vec<PriceObject>>,
  created_by: String,
  created_at: DateTime<Utc>,
}

impl Source
where
  Self: Sized,
{
  pub fn new(id: u32, data: SourceData, created_by: String) -> Self {
    Self {
      id,
      data,
      prices: HashMap::new(),
      created_by,
      created_at: Utc::now(),
    }
  }
  /// Update data object
  pub fn update_data(
    &mut self,
    name: String,
    address: String,
    email: Vec<String>,
    phone: Vec<String>,
  ) -> &Self {
    self.data = SourceData::new(name, address, email, phone);
    self
  }
  /// Add price object to a SKU
  pub fn add_price(&mut self, sku: u32, price_object: PriceObject) -> Option<&Vec<PriceObject>> {
    if let Some(prices) = self.prices.get_mut(&sku) {
      prices.push(price_object);
    } else {
      // Else insert a new sku price entry
      self.prices.insert(sku, vec![price_object]);
    }
    self.get_price_history(sku)
  }
  /// Get net retail price if available
  pub fn get_price(&self, sku: u32) -> Option<&PriceObject> {
    if let Some(prices) = self.prices.get(&sku) {
      return prices.last();
    }
    None
  }
  /// Get net retail price history if available
  pub fn get_price_history(&self, sku: u32) -> Option<&Vec<PriceObject>> {
    if let Some(prices) = self.prices.get(&sku) {
      return Some(prices);
    }
    None
  }
}

// This represents a source business data
pub struct SourceData {
  name: String,
  address: String,
  email: Vec<String>,
  phone: Vec<String>,
}

impl SourceData
where
  Self: Sized,
{
  pub fn new(name: String, address: String, email: Vec<String>, phone: Vec<String>) -> Self {
    Self {
      name,
      address,
      email,
      phone,
    }
  }
}
