use std::ops::Deref;

use datafusion::prelude::{SessionConfig, SessionContext};

use crate::{
    Backend, ReplDisplay,
    cli::{ConnectOpts, DatasetConn},
};

pub struct DataFusionBackend(SessionContext);

impl DataFusionBackend {
    pub fn new() -> Self {
        let mut config = SessionConfig::new();
        config.options_mut().catalog.information_schema = true;
        let ctx = SessionContext::new_with_config(config);
        Self(ctx)
    }
}

impl Default for DataFusionBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for DataFusionBackend {
    type DataFrame = datafusion::dataframe::DataFrame;

    async fn connect(&mut self, opts: &ConnectOpts) -> anyhow::Result<()> {
        match &opts.conn {
            DatasetConn::Postgres(_conn_str) => {
                println!("Postgres connection is not supported yet");
            }
            DatasetConn::Csv(filename) => {
                self.register_csv(&opts.name, filename, Default::default())
                    .await?;
            }
            DatasetConn::Parquet(filename) => {
                self.register_parquet(&opts.name, filename, Default::default())
                    .await?;
            }
            DatasetConn::NdJson(filename) => {
                self.register_json(&opts.name, filename, Default::default())
                    .await?;
            }
        }

        Ok(())
    }

    async fn list(&self) -> anyhow::Result<Self::DataFrame> {
        let sql = "select table_name, table_type from information_schema.tables where table_schema = 'public'";
        // let sql = "SHOW TABLES";
        let df = self.0.sql(sql).await?;
        Ok(df)
    }

    async fn describe(&self, name: &str) -> anyhow::Result<Self::DataFrame> {
        let df = self.0.sql(&format!("select * from {}", name)).await?;
        let df = df.describe().await?;
        Ok(df)
    }

    async fn schema(&self, name: &str) -> anyhow::Result<Self::DataFrame> {
        let df = self.0.sql(&format!("DESCRIBE {}", name)).await?;
        Ok(df)
    }

    async fn head(&self, name: &str, size: usize) -> anyhow::Result<Self::DataFrame> {
        let df = self
            .0
            .sql(&format!("SELECT * FROM {} LIMIT {}", name, size))
            .await?;
        Ok(df)
    }

    async fn sql(&self, sql: &str) -> anyhow::Result<Self::DataFrame> {
        let df = self.0.sql(sql).await?;
        Ok(df)
    }
}

impl Deref for DataFusionBackend {
    type Target = SessionContext;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ReplDisplay for datafusion::dataframe::DataFrame {
    async fn display(self) -> anyhow::Result<()> {
        self.show().await?;
        Ok(())
    }
}
