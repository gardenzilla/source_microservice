use chrono::prelude::*;
use packman::VecPackMember;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Price entity related to a SKU
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriceObject {
  net_price: u32,
  comment: String,
  created_at: DateTime<Utc>,
  created_by: String,
}

impl Default for PriceObject {
  fn default() -> Self {
    Self {
      net_price: 0,
      comment: "".into(),
      created_at: Utc::now(),
      created_by: "".into(),
    }
  }
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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Source {
  pub id: u32,
  pub data: SourceData,
  pub prices: HashMap<u32, Vec<PriceObject>>,
  pub created_by: String,
  pub created_at: DateTime<Utc>,
}

impl Default for Source {
  fn default() -> Self {
    Self {
      id: 0,
      data: SourceData::default(),
      prices: HashMap::new(),
      created_by: "".into(),
      created_at: Utc::now(),
    }
  }
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
  /// Get sku list available
  pub fn get_skus(&self) -> Vec<&u32> {
    self.prices.iter().map(|(sku, _)| sku).collect()
  }
  /// Get price list (SKU, Option<&PriceObject>)
  pub fn get_price_list(&self) -> Vec<(&u32, Option<&PriceObject>)> {
    self
      .prices
      .iter()
      .map(|(sku, prices)| (sku, prices.last()))
      .collect()
  }
}

impl VecPackMember for Source {
  type Out = u32;

  fn get_id(&self) -> &Self::Out {
    &self.id
  }
}

// This represents a source business data
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SourceData {
  pub name: String,
  pub address: String,
  pub email: Vec<String>,
  pub phone: Vec<String>,
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
