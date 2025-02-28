use std::sync::Arc;

use chrono::{Duration, Utc};
use crm_metadata::pb::{Content, MaterializeRequest};
use crm_send::pb::SendRequest;
use crm_user_stat::pb::QueryRequest;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};
use tracing::warn;

const CHANNEL_SIZE: usize = 1024;

use crate::{
    CrmService,
    pb::{WelcomeRequest, WelcomeResponse},
};

impl CrmService {
    pub async fn welcome(&self, req: WelcomeRequest) -> Result<Response<WelcomeResponse>, Status> {
        let request_id = req.id;
        let lower = Utc::now() - Duration::days(req.interval as _);
        let upper = lower + Duration::days(1);
        let query = QueryRequest::new_with_dt("created_at", lower, upper);
        let mut res_user_stats = self.user_stats.clone().query(query).await?.into_inner();

        let contents = self
            .metadata
            .clone()
            .materialize(MaterializeRequest::new_with_ids(&req.content_ids))
            .await?
            .into_inner();

        let contents = contents
            .filter_map(|v| async move { v.ok() })
            .collect::<Vec<Content>>()
            .await;

        let contents = Arc::new(contents);

        let (tx, rx) = mpsc::channel(CHANNEL_SIZE);

        let sender = self.config.application.sender_email.clone();
        tokio::spawn(async move {
            while let Some(Ok(user)) = res_user_stats.next().await {
                let sender = sender.clone();
                let contents = Arc::clone(&contents);
                let tx = tx.clone();

                let req = SendRequest::new("Welcome".to_string(), sender, &[user.email], &contents);
                if let Err(e) = tx.send(req).await {
                    warn!("Failed to send message: {:?}", e);
                }
            }
        });

        let reqs = ReceiverStream::new(rx);

        // =================================================================
        // let sender = self.config.application.sender_email.clone();
        // let reqs = res.filter_map(move |v| {
        //     let sender = sender.clone();
        //     let contents = contents.clone();
        //     async move {
        //         let v = v.ok()?;
        //         Some(gen_send_req("Welcome".to_string(), sender, v, &contents))
        //     }
        // });

        self.notification.clone().send(reqs).await?;

        Ok(Response::new(WelcomeResponse { id: request_id }))
    }
}
