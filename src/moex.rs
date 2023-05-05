use serde::{ Serialize, Deserialize };
use chrono::{ NaiveDateTime, NaiveDate, NaiveTime, DateTime, Utc };
use crate::chart::{
    TradeInterval,
    tradedata::{
        TradeItem, TradeData,
    }
};

#[derive(Debug)]
#[derive(Serialize,Deserialize)]
#[serde(untagged)]
pub enum MoexValue{
    String(String),
    Int(i32),
    Float(f32),
    Null,
}

#[derive(Debug)]
#[derive(Serialize,Deserialize)]
pub struct MoexHistory {
    pub columns: Vec<String>,
    pub data: Vec<Vec<MoexValue>>,
}

#[derive(Debug)]
#[derive(Serialize,Deserialize)]
pub struct MoexResponse {
    pub history: MoexHistory,
}

pub fn get_datetime(value: &MoexValue) -> Option<DateTime<Utc>> {
    if let MoexValue::String(datestring) = value {
        NaiveDate::parse_from_str(datestring,"%Y-%m-%d")
            .map(|d| NaiveDateTime::new(d, NaiveTime::default()))
            .map(|d| DateTime::from_utc(d,Utc))
            .ok()
    } else {
        None
    }
}

pub fn get_value(value: &MoexValue) -> Option<f32> {
    match value {
        MoexValue::String(_) |
        MoexValue::Null
            => None,
        MoexValue::Int(v)
            => Some(*v as f32),
        MoexValue::Float(v)
            => Some(*v),
    }
}

pub struct Moex {

}

impl Moex {
    pub async fn request_data(ticker: &str, from: NaiveDateTime) -> Result<TradeData, ()> {
        let url = format!("http://iss.moex.com/iss/history/engines/stock/markets/shares/boards/tqbr/securities/{}.json?from={}", ticker, from.format("%Y-%m-%d").to_string());

        let d: MoexResponse =
            reqwest::get(url)
                .await.unwrap()
                .json()
                .await.unwrap();
    
        let mut d_pos: Option<usize> = None;
        let mut h_pos: Option<usize> = None;
        let mut l_pos: Option<usize> = None;
        let mut o_pos: Option<usize> = None;
        let mut c_pos: Option<usize> = None;
        let mut v_pos: Option<usize> = None;
    
        for (idx, column) in d.history.columns.iter().enumerate() {
            match column.as_str() {
                "TRADEDATE" => d_pos = Some(idx),
                "HIGH"      => h_pos = Some(idx),
                "LOW"       => l_pos = Some(idx),
                "OPEN"      => o_pos = Some(idx),
                "CLOSE"     => c_pos = Some(idx),
                "VOLUME"    => v_pos = Some(idx),
                _ => (),
            }
        }
    
        let mut trade_data: TradeData = TradeData::new(TradeInterval::Day);
    
        if let Some(dpos) = d_pos {
            if let Some(hpos) = h_pos {
                if let Some(lpos) = l_pos {
                    if let Some(opos) = o_pos {
                        if let Some(cpos) = c_pos {
                            if let Some(vpos) = v_pos {
                                for dt in d.history.data {
                                    trade_data.add_item(
                                        TradeItem::new(
                                            get_datetime(&dt[dpos]).unwrap(),
                                            get_value(&dt[hpos]).unwrap(),
                                            get_value(&dt[lpos]).unwrap(),
                                            get_value(&dt[opos]).unwrap(),
                                            get_value(&dt[cpos]).unwrap(),
                                            get_value(&dt[vpos]).unwrap(),
                                        )
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(trade_data)
    }
}

#[test]
fn moex_check() {
    assert_eq!(NaiveDate::parse_from_str("2022-11-01", "%Y-%m-%d"),
           Ok(NaiveDate::from_ymd_opt(2022, 11, 1).unwrap()));
    assert_eq!(NaiveDateTime::parse_from_str("2022-11-01 00:00:00", "%Y-%m-%d %H:%M:%S"),
           Ok(NaiveDateTime::new(NaiveDate::from_ymd_opt(2022, 11, 1).unwrap(),NaiveTime::default())));
    let v: DateTime<Utc> = NaiveDate::parse_from_str("2022-11-01","%Y-%m-%d").map(|d| NaiveDateTime::new(d, NaiveTime::default())).map(|d| DateTime::from_utc(d,Utc)).unwrap();
    assert_eq!(v, DateTime::parse_from_rfc2822("Tue, 01 Nov 2022 00:00:00 GMT").unwrap());
}
