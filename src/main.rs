use gzlib::prelude::*;
use gzlib::proto::source::source_server::*;
use gzlib::proto::source::*;
use packman::*;
use prelude::{ServiceError, ServiceResult};
use source_server::Source;
use std::{collections::HashMap, env, path::PathBuf};
use tokio::sync::{oneshot, Mutex};
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
      created_at: res.created_by,
      created_by: res.created_at.to_rfc3339(),
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

  type GetAllSourcesStream = tokio::sync::mpsc::Receiver<Result<SourceObject, Status>>;

  async fn get_all_sources(
    &self,
    request: Request<()>,
  ) -> Result<Response<Self::GetAllSourcesStream>, Status> {
    // Create channel for stream response
    let (mut tx, rx) = tokio::sync::mpsc::channel(100);

    // Get resources as Vec<SourceObject>
    let res = self.get_all_sources().await?;

    // Send the result items through the channel
    for sobject in res {
      tx.send(Ok(sobject))
        .await
        .map_err(|_| Status::internal("Error while sending sources over channel"))?;
    }

    // Send back the receiver
    Ok(Response::new(rx))
  }

  type GetPricesStream = tokio::sync::mpsc::Receiver<Result<GetPriceInfoResponse, Status>>;

  async fn get_prices(
    &self,
    request: Request<GetPricesRequest>,
  ) -> Result<Response<Self::GetPricesStream>, Status> {
    todo!()
  }

  async fn add_price_info(
    &self,
    request: Request<AddPriceInfoRequest>,
  ) -> Result<Response<GetPriceInfoHistoryResponse>, Status> {
    todo!()
  }

  type GetPriceInfoStream = tokio::sync::mpsc::Receiver<Result<GetPriceInfoResponse, Status>>;

  async fn get_price_info(
    &self,
    request: Request<GetPriceInfoRequest>,
  ) -> Result<Response<Self::GetPriceInfoStream>, Status> {
    todo!()
  }

  type GetPriceInfoHistoryStream =
    tokio::sync::mpsc::Receiver<Result<GetPriceInfoHistoryResponse, Status>>;

  async fn get_price_info_history(
    &self,
    request: Request<GetPriceInfoHistoryRequest>,
  ) -> Result<Response<Self::GetPriceInfoHistoryStream>, Status> {
    todo!()
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
