use anyhow::{bail, Context, Result};
use financeapi::{FinanceapiAutocomplete, FinanceapiConnector, FinanceapiQuote};

#[derive(Debug, Default)]
pub struct FinanceProvider {
    connector: FinanceapiConnector,
    key: Option<String>,
}

impl FinanceProvider {
    fn check_key(&self) -> Result<()> {
        if self.key.is_none() {
            bail!("key not set");
        }

        Ok(())
    }

    pub fn new(key: &str) -> Self {
        if key.is_empty() {
            Self::default()
        } else {
            Self {
                connector: FinanceapiConnector::new(key),
                key: Some(key.into()),
            }
        }
    }

    pub fn search(&self, symbol: &str) -> Result<Vec<FinanceapiAutocomplete>> {
        self.check_key()?;

        Ok(tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(self.connector.autocomplete(symbol))?)
    }

    pub fn get_quote(&self, symbol: &str) -> Result<FinanceapiQuote> {
        self.check_key()?;

        Ok(tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(self.connector.quote(symbol))?)
    }

    pub fn get_latest_price(&self, symbol: &str) -> Result<f64> {
        self.check_key()?;

        self.get_quote(symbol)?
            .regular_market_price
            .context("Unable to fetch latest price")
    }
}
