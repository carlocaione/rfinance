use crate::{data::Data, finance::FinanceProvider, table};
use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "")]
pub enum Command {
    Search {
        symbol: String,
    },
    Info {
        symbol: String,
    },
    Conf {
        #[arg(short, long)]
        reset: bool,
        #[arg(short, long)]
        set_key: Option<String>,
    },
    Add {
        symbol: String,
        quantity: u32,
        price: Option<f64>,
        date: Option<String>,
    },
    Delete {
        symbol: String,
        #[arg(short, long)]
        index: Option<usize>,
    },
    Show,
}

pub struct Cmd<'a> {
    data: &'a mut Data,
    finance: &'a mut FinanceProvider,
}

impl<'a> Cmd<'a> {
    pub fn new(data: &'a mut Data, finance: &'a mut FinanceProvider) -> Self {
        Self { data, finance }
    }

    pub fn parse(&mut self, command: Command) -> Result<()> {
        match command {
            Command::Conf { reset, set_key } => self.conf(reset, set_key),
            Command::Search { symbol } => self.search(symbol),
            Command::Info { symbol } => self.info(symbol),
            Command::Add {
                symbol,
                quantity,
                price,
                date,
            } => self.add(symbol, quantity, price, date),
            Command::Show => self.show(),
            Command::Delete { symbol, index } => self.delete(symbol, index),
        }
    }

    pub fn conf(&mut self, reset: bool, set_key: Option<String>) -> Result<()> {
        if reset {
            self.data.reset()?;
            *self.finance = FinanceProvider::default();
        } else if let Some(key) = set_key {
            self.data.api_key = key;
            self.data.save()?;
            *self.finance = FinanceProvider::new(&self.data.api_key);
        }

        println!("API key: {}", self.data.api_key);
        println!("DATA file: {}", self.data.data_file.display());

        Ok(())
    }

    pub fn search(&self, symbol: String) -> Result<()> {
        table::search(self.finance, symbol)
    }

    pub fn info(&self, symbol: String) -> Result<()> {
        table::info(self.finance, symbol)
    }

    pub fn add(
        &mut self,
        symbol: String,
        quantity: u32,
        price: Option<f64>,
        date: Option<String>,
    ) -> Result<()> {
        let date = date.map_or_else(
            || Ok(Utc::now().date_naive()),
            |d| NaiveDate::parse_from_str(&d, "%d/%m/%y").context("Wrong date format: dd/mm/yy"),
        )?;

        let price = price.map_or_else(|| self.finance.get_latest_price(&symbol), Ok)?;

        self.data.add(symbol, quantity, price, date)
    }

    pub fn show(&self) -> Result<()> {
        table::show_portfolio(self.finance, &self.data.portfolio)
    }

    pub fn delete(&mut self, symbol: String, index: Option<usize>) -> Result<()> {
        self.data.delete(symbol, index)
    }
}
