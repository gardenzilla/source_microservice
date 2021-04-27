use gzlib::proto::source::source_server::*;
use gzlib::proto::source::*;
use packman::*;
use prelude::{ServiceError, ServiceResult};
use std::{env, path::PathBuf};
use tokio::sync::{oneshot, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

mod prelude;
mod source;

struct SourceService {
  sources: Mutex<VecPack<source::Source>>,
}

impl SourceService {
  fn new(db: VecPack<source::Source>) -> Self {
    Self {
      sources: Mutex::new(db),
    }
  }
  // Get next id
  // Iterate over all the IDs and returns the max ID
  // value + 1
  async fn next_id(&self) -> u32 {
    let mut latest_id: u32 = 0;
    self.sources.lock().await.iter().for_each(|source| {
      let id: u32 = *source.unpack().get_id();
      if id > latest_id {
        latest_id = id;
      }
    });
    latest_id + 1
  }
  async fn create_source(&self, r: CreateSourceRequest) -> ServiceResult<SourceObject> {
    let new_source_id = self.next_id().await;
    let new_source = source::Source::new(
      new_source_id,
      source::SourceData::new(r.name, r.address, r.email, r.phone),
      r.created_by,
    );
    self.sources.lock().await.insert(new_source.clone())?;

    // Get the last inserted (this) source
    let new_source = self
      .sources
      .lock()
      .await
      .last()
      .ok_or(ServiceError::internal_error("Az új source nem található!"))?
      .unpack()
      .clone();

    // Return this as SourceObject
    Ok(SourceObject {
      id: new_source.id,
      name: new_source.data.name,
      address: new_source.data.address,
      email: new_source.data.email,
      phone: new_source.data.phone,
      created_at: new_source.created_at.to_rfc3339(),
      created_by: new_source.created_by,
    })
  }
  async fn get_source(&self, r: GetSourceRequest) -> ServiceResult<SourceObject> {
    // Try find the requested source
    let res = self
      .sources
      .lock()
      .await
      .find_id(&r.source_id)
      .map_err(|_| ServiceError::not_found("A megadott source nem található"))?
      .unpack()
      .clone();

    // Returns SourceObject
    Ok(SourceObject {
      id: res.id,
      name: res.data.name,
      address: res.data.address,
      email: res.data.email,
      phone: res.data.phone,
      created_at: res.created_at.to_rfc3339(),
      created_by: res.created_by,
    })
  }

  async fn update_source(&self, r: SourceObject) -> ServiceResult<SourceObject> {
    // Try find source as mut
    match self.sources.lock().await.find_id_mut(&r.id) {
      Ok(source) => {
        // Try update source data
        let res = source
          .as_mut()
          .unpack()
          .update_data(r.name, r.address, r.email, r.phone)
          .clone();

        // Return result updated source data
        Ok(SourceObject {
          id: res.id,
          name: res.data.name,
          address: res.data.address,
          email: res.data.email,
          phone: res.data.phone,
          created_at: res.created_at.to_rfc3339(),
          created_by: res.created_by,
        })
      }
      Err(_) => Err(ServiceError::not_found("A megadott source nem található!")),
    }
  }

  async fn get_all_sources(&self) -> ServiceResult<Vec<SourceObject>> {
    let res = self
      .sources
      .lock()
      .await
      .iter()
      .map(|s| {
        let source = s.unpack().clone();
        SourceObject {
          id: source.id,
          name: source.data.name,
          address: source.data.address,
          email: source.data.email,
          phone: source.data.phone,
          created_at: source.created_at.to_rfc3339(),
          created_by: source.created_by,
        }
      })
      .collect::<Vec<SourceObject>>();
    Ok(res)
  }

  async fn get_prices(&self, r: GetPricesRequest) -> ServiceResult<Vec<GetPriceInfoResponse>> {
    let mut res: Vec<GetPriceInfoResponse> = Vec::new();
    self
      .sources
      .lock()
      .await
      .find_id(&r.source_id)?
      .unpack()
      .prices
      .iter()
      .for_each(|(sku, price_vec)| {
        // If we can get the last price of a SKU
        if let Some(latest_price) = price_vec.last() {
          res.push(GetPriceInfoResponse {
            source_id: r.source_id,
            sku: *sku,
            latest_price: Some(PriceObject {
              net_price: latest_price.net_price,
              comment: latest_price.comment.to_owned(),
              created_at: latest_price.created_at.to_rfc3339(),
              created_by: latest_price.created_by.to_owned(),
            }),
          });
        }
      });
    Ok(res)
  }

  async fn add_price_info(&self, r: AddPriceInfoRequest) -> ServiceResult<Vec<PriceObject>> {
    let res = self
      .sources
      .lock()
      .await
      .find_id_mut(&r.source_id)?
      .as_mut()
      .unpack()
      .add_price(
        r.sku,
        source::PriceObject::new(r.net_price, r.comment, r.created_by),
      )
      .cloned()
      .ok_or(ServiceError::internal_error(
        "Error while getting price history after added new price",
      ))?;

    let res = res
      .iter()
      .map(|pobject| PriceObject {
        net_price: pobject.net_price,
        comment: pobject.comment.to_owned(),
        created_at: pobject.created_at.to_rfc3339(),
        created_by: pobject.created_by.to_owned(),
      })
      .collect::<Vec<PriceObject>>();

    Ok(res)
  }

  async fn get_price_info(
    &self,
    r: GetPriceInfoRequest,
  ) -> ServiceResult<Vec<GetPriceInfoResponse>> {
    let mut res: Vec<GetPriceInfoResponse> = Vec::new();
    self.sources.lock().await.iter().for_each(|s| {
      let s = s.unpack();
      if let Some(price) = s.get_price(r.sku) {
        res.push(GetPriceInfoResponse {
          source_id: s.id,
          sku: r.sku,
          latest_price: Some(PriceObject {
            net_price: price.net_price,
            comment: price.comment.to_owned(),
            created_at: price.created_at.to_rfc3339(),
            created_by: price.created_by.to_owned(),
          }),
        });
      }
    });
    Ok(res)
  }

  async fn get_price_info_history(
    &self,
    r: GetPriceInfoHistoryRequest,
  ) -> ServiceResult<Vec<PriceObject>> {
    match self
      .sources
      .lock()
      .await
      .find_id(&r.source)?
      .unpack()
      .prices
      .get(&r.sku)
    {
      Some(prices) => {
        let res = prices
          .iter()
          .map(|p| PriceObject {
            net_price: p.net_price,
            comment: p.comment.to_owned(),
            created_at: p.created_at.to_rfc3339(),
            created_by: p.created_by.to_owned(),
          })
          .collect::<Vec<PriceObject>>();
        Ok(res)
      }
      None => Err(ServiceError::not_found("A kért source / SKU nem létezik")),
    }
  }
}

#[tonic::async_trait]
impl gzlib::proto::source::source_server::Source for SourceService {
  async fn create_source(
    &self,
    request: Request<CreateSourceRequest>,
  ) -> Result<Response<SourceObject>, Status> {
    Ok(Response::new(
      self.create_source(request.into_inner()).await?,
    ))
  }

  async fn get_source(
    &self,
    request: Request<GetSourceRequest>,
  ) -> Result<Response<SourceObject>, Status> {
    let result = self.get_source(request.into_inner()).await?;
    Ok(Response::new(result))
  }

  async fn update_source(
    &self,
    request: Request<SourceObject>,
  ) -> Result<Response<SourceObject>, Status> {
    let res = self.update_source(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  type GetAllSourcesStream = ReceiverStream<Result<SourceObject, Status>>;

  async fn get_all_sources(
    &self,
    _request: Request<()>,
  ) -> Result<Response<Self::GetAllSourcesStream>, Status> {
    // Create channel for stream response
    let (mut tx, rx) = tokio::sync::mpsc::channel(100);

    // Get resources as Vec<SourceObject>
    let res = self.get_all_sources().await?;

    // Send the result items through the channel
    tokio::spawn(async move {
      for ots in res.into_iter() {
        tx.send(Ok(ots)).await.unwrap();
      }
    });

    // Send back the receiver
    Ok(Response::new(ReceiverStream::new(rx)))
  }

  type GetPricesStream = ReceiverStream<Result<GetPriceInfoResponse, Status>>;

  async fn get_prices(
    &self,
    request: Request<GetPricesRequest>,
  ) -> Result<Response<Self::GetPricesStream>, Status> {
    // Create channel for stream response
    let (mut tx, rx) = tokio::sync::mpsc::channel(100);

    // Get prices
    let res = self.get_prices(request.into_inner()).await?;

    // Send prices over channel
    tokio::spawn(async move {
      for ots in res.into_iter() {
        tx.send(Ok(ots)).await.unwrap();
      }
    });

    // Send back the receiver
    Ok(Response::new(ReceiverStream::new(rx)))
  }

  type AddPriceInfoStream = ReceiverStream<Result<PriceObject, Status>>;

  async fn add_price_info(
    &self,
    request: Request<AddPriceInfoRequest>,
  ) -> Result<Response<Self::AddPriceInfoStream>, Status> {
    // Create channel for stream response
    let (mut tx, rx) = tokio::sync::mpsc::channel(100);

    let res = self.add_price_info(request.into_inner()).await?;

    tokio::spawn(async move {
      for ots in res.into_iter() {
        tx.send(Ok(ots)).await.unwrap();
      }
    });

    // Send back the receiver
    Ok(Response::new(ReceiverStream::new(rx)))
  }

  type GetPriceInfoStream = ReceiverStream<Result<GetPriceInfoResponse, Status>>;

  async fn get_price_info(
    &self,
    request: Request<GetPriceInfoRequest>,
  ) -> Result<Response<Self::GetPriceInfoStream>, Status> {
    // Create channel for stream response
    let (mut tx, rx) = tokio::sync::mpsc::channel(100);

    // Get prices
    let res = self.get_price_info(request.into_inner()).await?;

    // Send prices over channel
    tokio::spawn(async move {
      for ots in res.into_iter() {
        tx.send(Ok(ots)).await.unwrap();
      }
    });

    // Send back the receiver
    Ok(Response::new(ReceiverStream::new(rx)))
  }

  type GetPriceInfoHistoryStream = ReceiverStream<Result<PriceObject, Status>>;

  async fn get_price_info_history(
    &self,
    request: Request<GetPriceInfoHistoryRequest>,
  ) -> Result<Response<Self::GetPriceInfoHistoryStream>, Status> {
    // Create channel for stream response
    let (mut tx, rx) = tokio::sync::mpsc::channel(100);

    let res = self.get_price_info_history(request.into_inner()).await?;

    tokio::spawn(async move {
      for ots in res.into_iter() {
        tx.send(Ok(ots)).await.unwrap();
      }
    });

    // Send back the receiver
    Ok(Response::new(ReceiverStream::new(rx)))
  }
}

#[tokio::main]
async fn main() -> prelude::ServiceResult<()> {
  let db: VecPack<source::Source> =
    VecPack::load_or_init(PathBuf::from("data/source")).expect("Error while loading source db");

  let source_service = SourceService::new(db);

  let addr = env::var("SERVICE_ADDR_SOURCE")
    .unwrap_or("[::1]:50062".into())
    .parse()
    .unwrap();

  // Create shutdown channel
  let (tx, rx) = oneshot::channel();

  // Spawn the server into a runtime
  tokio::task::spawn(async move {
    Server::builder()
      .add_service(SourceServer::new(source_service))
      .serve_with_shutdown(addr, async { rx.await.unwrap() })
      .await
  });

  tokio::signal::ctrl_c().await.unwrap();

  println!("SIGINT");

  // Send shutdown signal after SIGINT received
  let _ = tx.send(());

  Ok(())
}
