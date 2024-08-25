use anyhow::Result;
use core::fmt;
use derive_more::derive::FromStr;
use financeapi::{FinanceapiAutocomplete, FinanceapiQuote};
use owo_colors::OwoColorize;
use std::{iter, str::FromStr};
use tabled::{
    settings::{
        object::{Columns, Object, Rows},
        style::HorizontalLine,
        Format, Style,
    },
    Table, Tabled,
};

use crate::{
    data::{Performance, Portfolio},
    finance::FinanceProvider,
};

#[derive(FromStr, Debug, Default)]
struct Symbol(String);

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.bold().red())
    }
}

#[derive(FromStr, Debug, Default)]
struct Price(f64);

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0.bold())
    }
}

#[derive(FromStr, Debug, Default)]
struct Gain(f64);

impl fmt::Display for Gain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_sign_positive() {
            let s = format!("+{:.2}", self.0);
            write!(f, "{}", s.green())
        } else {
            write!(f, "{:.2}", self.0.red())
        }
    }
}

#[derive(FromStr, Debug, Default)]
struct PercGain(f64);

impl fmt::Display for PercGain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_sign_positive() {
            let s = format!("+{:.2}%", self.0);
            write!(f, "{}", s.green())
        } else {
            let s = format!("{:.2}%", self.0);
            write!(f, "{}", s.red())
        }
    }
}

#[derive(FromStr, Debug, Default)]
struct Value(f64);

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_sign_positive() {
            write!(f, "{:.2}", self.0.green())
        } else {
            write!(f, "{:.2}", self.0.red())
        }
    }
}

#[derive(Tabled)]
struct TableSearch {
    asset: String,
    #[tabled(rename = "ticker")]
    symbol: Symbol,
    exchange: String,
    description: String,
}

impl From<FinanceapiAutocomplete> for TableSearch {
    fn from(value: FinanceapiAutocomplete) -> Self {
        Self {
            asset: value.type_disp,
            symbol: Symbol::from_str(&value.symbol).unwrap_or_default(),
            exchange: value.exch_disp,
            description: value.name,
        }
    }
}

pub fn search(finance: &FinanceProvider, symbol: String) -> Result<()> {
    let quotes = finance.search(&symbol)?;
    let content = quotes
        .into_iter()
        .map(TableSearch::from)
        .collect::<Vec<_>>();

    let mut table = Table::new(content);

    table.with(Style::sharp()).modify(
        Columns::last().intersect(Rows::new(1..)),
        Format::content(|s| s.green().to_string()),
    );

    println!("{table}");

    Ok(())
}

#[derive(Tabled)]
struct TableInfo {
    asset: String,
    #[tabled(rename = "ticker")]
    symbol: Symbol,
    currency: String,
    price: Price,
    #[tabled(rename = "day gain")]
    day_gain: Gain,
    #[tabled(rename = "day gain (%)")]
    day_gain_perc: PercGain,
}

impl From<FinanceapiQuote> for TableInfo {
    fn from(value: FinanceapiQuote) -> Self {
        Self {
            asset: value.type_disp.unwrap_or_default(),
            symbol: Symbol::from_str(&value.symbol).unwrap_or_default(),
            currency: value.currency.unwrap_or_default(),
            price: Price(value.regular_market_price.unwrap_or_default()),
            day_gain: Gain(value.regular_market_change.unwrap_or_default()),
            day_gain_perc: PercGain(value.regular_market_change_percent.unwrap_or_default()),
        }
    }
}

pub fn info(finance: &FinanceProvider, symbol: String) -> Result<()> {
    let quote = finance.get_quote(&symbol)?;
    let content = vec![TableInfo::from(quote)];

    let mut table = Table::new(content);
    table.with(Style::sharp());

    println!("{table}");

    Ok(())
}

#[derive(Tabled)]
struct TablePortfolio {
    invested: Value,
    gain: Gain,
    #[tabled(rename = "gain (%)")]
    gain_perc: PercGain,
    #[tabled(rename = "current value")]
    value: Value,
}

impl From<&Performance> for TablePortfolio {
    fn from(value: &Performance) -> Self {
        Self {
            invested: Value(value.invested_value),
            gain: Gain(value.gain),
            gain_perc: PercGain(value.gain_perc),
            value: Value(value.latest_value),
        }
    }
}

#[derive(Tabled)]
struct TableAsset {
    #[tabled(rename = "")]
    index: String,
    #[tabled(rename = "")]
    header: String,
    price: Price,
    quantity: u32,
    invested: Value,
    gain: Gain,
    #[tabled(rename = "gain (%)")]
    gain_perc: PercGain,
    #[tabled(rename = "current value")]
    value: Value,
}

impl TableAsset {
    pub fn new(header: &str, price: f64, performance: &Performance, index: String) -> TableAsset {
        Self {
            index,
            header: header.to_owned(),
            price: Price(price),
            quantity: performance.quantity,
            invested: Value(performance.invested_value),
            gain: Gain(performance.gain),
            gain_perc: PercGain(performance.gain_perc),
            value: Value(performance.latest_value),
        }
    }
}

pub fn show_portfolio(finance: &FinanceProvider, portfolio: &Portfolio) -> Result<()> {
    let portfolio_performance = portfolio.performance(finance)?;
    let content = vec![TablePortfolio::from(&portfolio_performance)];

    let mut table = Table::new(content);
    table.with(Style::sharp());
    println!("{table}");

    for asset in portfolio.asset.values() {
        let latest = finance.get_latest_price(&asset.symbol)?;
        let asset_performance = asset.performance(finance)?;

        let v = iter::once(TableAsset::new(
            &asset.symbol,
            latest,
            &asset_performance,
            format!(
                "{:.2}%",
                asset_performance.latest_value / portfolio_performance.latest_value * 100f64
            ),
        ))
        .chain(asset.op.iter().enumerate().map(|(index, op)| {
            TableAsset::new(
                &op.date.format("%d/%m/%y").to_string(),
                op.price,
                &op.performance(finance, Some(latest)).unwrap(),
                (index + 1).to_string(),
            )
        }))
        .collect::<Vec<_>>();

        let mut table = Table::new(v);
        table
            .with(Style::sharp().horizontals([
                (1, HorizontalLine::inherit(Style::modern())),
                (2, HorizontalLine::inherit(Style::modern())),
            ]))
            .modify(
                Columns::new(1..2).intersect(Rows::new(1..2)),
                Format::content(|s| s.to_ascii_uppercase().red().bold().to_string()),
            );

        println!("{table}");
    }

    Ok(())
}
