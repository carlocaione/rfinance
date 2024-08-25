use anyhow::{bail, Context, Result};
use chrono::NaiveDate;
use directories_next::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io, ops::Add, path::PathBuf};

use crate::finance;

static PROGNAME: &str = env!("CARGO_PKG_NAME");

#[derive(Debug, Default)]
pub struct Performance {
    pub invested_value: f64,
    pub latest_value: f64,
    pub gain: f64,
    pub gain_perc: f64,
    pub quantity: u32,
}

impl Performance {
    pub fn new(quantity: u32, buying_price: f64, current_price: f64) -> Self {
        let invested_value = quantity as f64 * buying_price;
        let latest_value = quantity as f64 * current_price;
        let gain = latest_value - invested_value;
        let gain_perc = gain / invested_value * 100f64;

        Self {
            invested_value,
            latest_value,
            gain,
            gain_perc,
            quantity,
        }
    }
}

impl Add for Performance {
    type Output = Performance;

    fn add(self, rhs: Self) -> Self::Output {
        let invested_value = self.invested_value + rhs.invested_value;
        let latest_value = self.latest_value + rhs.latest_value;
        let gain = latest_value - invested_value;
        let gain_perc = gain / invested_value * 100f64;
        let quantity = self.quantity + rhs.quantity;

        Self {
            invested_value,
            latest_value,
            gain,
            gain_perc,
            quantity,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AssetOp {
    pub symbol: String,
    pub quantity: u32,
    pub price: f64,
    pub date: NaiveDate,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Asset {
    pub symbol: String,
    pub op: Vec<AssetOp>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Portfolio {
    pub asset: HashMap<String, Asset>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Data {
    pub api_key: String,
    pub portfolio: Portfolio,

    #[serde(skip)]
    pub data_file: PathBuf,
}

impl AssetOp {
    pub fn performance(
        &self,
        finance: &finance::FinanceProvider,
        current_price: Option<f64>,
    ) -> Result<Performance> {
        let current_price = match current_price {
            None => finance.get_latest_price(&self.symbol)?,
            Some(p) => p,
        };

        Ok(Performance::new(self.quantity, self.price, current_price))
    }
}

impl Asset {
    pub fn performance(&self, finance: &finance::FinanceProvider) -> Result<Performance> {
        let latest = finance.get_latest_price(&self.symbol)?;

        self.op.iter().try_fold(Performance::default(), |acc, x| {
            let p = x.performance(finance, Some(latest))?;
            Ok::<Performance, anyhow::Error>(acc + p)
        })
    }
}

impl Portfolio {
    pub fn performance(&self, finance: &finance::FinanceProvider) -> Result<Performance> {
        self.asset
            .values()
            .try_fold(Performance::default(), |acc, x| {
                let p = x.performance(finance)?;
                Ok::<Performance, anyhow::Error>(acc + p)
            })
    }
}

impl Data {
    pub fn reset(&mut self) -> Result<()> {
        *self = Self {
            data_file: self.data_file.clone(),
            ..Default::default()
        };

        self.save()
    }

    pub fn save(&self) -> Result<()> {
        Ok(fs::write(&self.data_file, toml::to_string(&self)?)?)
    }

    pub fn load() -> Result<Self> {
        let data_dir = ProjectDirs::from("", "", PROGNAME)
            .context("Failed to get project directory")?
            .data_dir()
            .to_owned();

        let mut data_file = data_dir.join(PROGNAME);
        data_file.set_extension("dat");

        fs::create_dir_all(&data_dir)?;

        let mut data = match fs::read_to_string(&data_file) {
            Ok(content) => toml::from_str(&content)?,
            Err(err) if err.kind() == io::ErrorKind::NotFound => Data::default(),
            Err(_) => bail!("Error while opening {}", data_file.display()),
        };

        data.data_file = data_file;
        data.save()?;

        Ok(data)
    }

    pub fn add(
        &mut self,
        symbol: String,
        quantity: u32,
        price: f64,
        date: NaiveDate,
    ) -> Result<()> {
        let asset = self.portfolio.asset.entry(symbol.clone()).or_default();

        asset.op.push(AssetOp {
            symbol: symbol.clone(),
            quantity,
            price,
            date,
        });
        asset.symbol = symbol;

        self.save()
    }

    pub fn delete(&mut self, symbol: String, index: Option<usize>) -> Result<()> {
        match index {
            None => {
                self.portfolio
                    .asset
                    .remove(&symbol)
                    .context("Symbol not found")?;
            }
            Some(index) => {
                let op = &mut self
                    .portfolio
                    .asset
                    .get_mut(&symbol)
                    .context("Symbol not found")?
                    .op;

                if index == 0 || index > op.len() {
                    bail!("Index out of bounds");
                }

                op.remove(index - 1);
            }
        }

        self.save()
    }
}
