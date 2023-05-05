use core::slice::Iter;
use chrono::{ DateTime, Utc, };
use crate::chart::{ Period, RangeF32, Frame, TradeInterval };

pub struct Hlocv {
    pub h: f32,
    pub l: f32,
    pub o: f32,
    pub c: f32,
    pub v: f32,
}

impl Hlocv {
    pub fn new(h: f32, l: f32, o: f32, c: f32, v: f32) -> Hlocv {
        Hlocv {
            h,
            l,
            o,
            c,
            v,
        }
    }
}

pub struct TradeItem {
    date: DateTime<Utc>,
    hlocv: Hlocv,
}

impl TradeItem {
    pub fn new(d: DateTime<Utc>, h: f32, l: f32, o: f32, c: f32, v: f32) -> TradeItem {
        TradeItem { date: d, hlocv: Hlocv::new(h ,l, o, c, v) }
    }
    pub fn _timestamp(&self) -> i64 {
        self.date.timestamp()
    }
    pub fn hlocv(&self) -> &Hlocv {
        &self.hlocv
    }
}

pub struct TradeItemPositioned<'a> {
    pub item: &'a TradeItem,
    pub position: u32,
}

impl<'a> TradeItemPositioned<'a> {
    pub fn new(item: &TradeItem, position: u32) -> TradeItemPositioned {
        TradeItemPositioned {
            item,
            position
        }
    }
}

pub struct TradeData {
    _interval: TradeInterval,
    items: Vec<TradeItem>,
    period: Period<Utc>,
    range: RangeF32,
}

impl TradeData {
    pub fn new(i: TradeInterval)-> TradeData {
        TradeData {
            _interval: i,
            items: Vec::new(),
            period: Period::<Utc>::default(),
            range: RangeF32::new_with_max_rev(),
        }
    }
    pub fn add_item(&mut self, item: TradeItem) {
        self.period.consider(item.date);
        self.range.consider(item.hlocv.l, item.hlocv.h);
        self.items.push(item);
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn iter_data(&self) -> Iter<TradeItem> {
        self.items.iter()
    }
    pub fn _interval(&self) -> &TradeInterval {
        &self._interval
    }
    fn _period(&self) -> &Period<Utc> {
        &self.period
    }
    pub fn range(&self) -> &RangeF32 {
        &self.range
    }
}

pub fn union(a: &Frame, b: &Frame) -> Frame {
    Frame::new(
        if a.range_x().start() < b.range_x().start() { a.range_x().start() } else { b.range_x().start() }
        ..
        if a.range_x().end() > b.range_x().end() { a.range_x().end() } else { b.range_x().end() }
        ,
        if a.range_y().start() < b.range_y().start() { a.range_y().start() } else { b.range_y().start() }
        ..
        if a.range_y().end() > b.range_y().end() { a.range_y().end() } else { b.range_y().end() }
    )
}

