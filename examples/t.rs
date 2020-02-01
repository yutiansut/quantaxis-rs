extern crate csv;
extern crate ndarray;
extern crate ndarray_csv;
extern crate num_traits;
extern crate serde;
extern crate stopwatch;

use std::borrow::BorrowMut;
use std::io;

use ndarray::{array, stack};
use stopwatch::Stopwatch;

use quantaxis_rs::{indicators, Next, qaaccount, qadata, qafetch, qaindicator, qaorder, transaction};
use quantaxis_rs::indicators::{
    BollingerBands, EfficiencyRatio, ExponentialMovingAverage, FastStochastic, HHV, LLV,
    Maximum, Minimum, MoneyFlowIndex, MovingAverage,
    MovingAverageConvergenceDivergence, OnBalanceVolume, RateOfChange, RelativeStrengthIndex, SimpleMovingAverage,
    SlowStochastic, StandardDeviation, TrueRange,
};
use quantaxis_rs::qaaccount::QA_Account;
use quantaxis_rs::qaorder::QA_Postions;

pub fn backtest() {
    let priceoffset = 1;
    let lossP = 1.3;
    let K1 = 20;
    let K2 = 20;
    let n1: usize = 30;

    let count1 = 0;
    let mut HAE: f64 = 0 as f64;
    let mut LAE: f64 = 0 as f64;
    let TrailingStart1 = 90;
    let TrailingStop1 = 10;
    let init_data = qafetch::BAR {
        code: "".to_string(),
        datetime: "".to_string(),
        open: 0.0,
        high: 0.0,
        low: 0.0,
        close: 0.0,
        volume: 0.0,
    };
    let mut acc = qaaccount::QA_Account::new("BacktestAccount");
    acc.init_h("RB2005");
    let mut llv_i = LLV::new(3).unwrap();
    let mut hhv_i = HHV::new(3).unwrap();
    let mut ma = MovingAverage::new(n1 as u32).unwrap();
    let mut rdr = csv::Reader::from_reader(io::stdin());
    let mut lastbar = qafetch::BAR{
        code: "".to_string(),
        datetime: "".to_string(),
        open: 0.0,
        high: 0.0,
        low: 0.0,
        close: 0.0,
        volume: 0.0
    };
    for result in rdr.deserialize() {
        let bar: qafetch::BAR = result.unwrap() ;
        let ind_llv = llv_i.next(bar.low);
        let ind_hhv = hhv_i.next(bar.high);
        let ind_ma = ma.next(bar.close);
        let crossOver = bar.high > hhv_i.cached[1] && lastbar.high < hhv_i.cached[1];

        let crossUnder = bar.low < llv_i.cached[1] && lastbar.low > llv_i.cached[1];

        let cond1 = ma.cached[n1 -1]> ma.cached[n1 -2] &&
                        ma.cached[n1 -2]> ma.cached[n1 -3] &&
            ma.cached[n1 - 3] > ma.cached[n1 - 4] &&
            ma.cached[n1 - 4] > ma.cached[n1 - 5];


        let cond2 = ma.cached[n1 - 1] < ma.cached[n1 - 2] &&
            ma.cached[n1 - 2] < ma.cached[n1 - 3] &&
            ma.cached[n1 - 3] < ma.cached[n1 - 4] &&
            ma.cached[n1 - 4] < ma.cached[n1 - 5];

        let code = bar.code.as_ref();
        if (acc.get_position_long(code) > 0.0 || acc.get_position_short(code) > 0.0) {
            HAE = lastbar.high;
            LAE = lastbar.low;
        }

        if (acc.get_position_long(code) == 0 as f64 && acc.get_position_short(code) == 0 as f64) {
            if crossOver && cond1 {
                acc.buy_open(bar.code.as_ref(), 10.0, bar.datetime.as_ref(), bar.close);
            }
            if crossUnder && cond2 {
                acc.sell_open(bar.code.as_ref(), 10.0, bar.datetime.as_ref(), bar.close);
            }
        }
        if (acc.get_position_long(code) > 0 as f64 && acc.get_position_short(code) == 0 as f64) {
            println!("当前多单持仓");

            let mut stopLine: f64 = acc.get_open_price_long(code) * (100.0 - lossP) / 100 as f64;
            if (HAE >= (acc.get_open_price_long(code) * (1 + TrailingStart1 / 1000) as f64)) {
                stopLine = (HAE * (1 - TrailingStop1 / 1000) as f64) as f64;
            }

            if (crossUnder && cond2) {
                acc.sell_close(code, 10.0, bar.datetime.as_ref(), bar.open);
            }
            if (bar.low < stopLine) {
                acc.sell_close(code, 10.0, bar.datetime.as_ref(), bar.open);
            }
        }
        if (acc.get_position_short(code) > 0 as f64 && acc.get_position_long(code) == 0 as f64) {
            println!("当前空单持仓 {:#?}", acc.get_position_short(code));
            let mut stopLine: f64 = acc.get_open_price_short(code) * (100.0 + lossP) / 100 as f64;

            if (LAE >= (acc.get_open_price_short(code) * (1 - TrailingStart1 / 1000) as f64)) {
                stopLine = (LAE * (1 + TrailingStop1 / 1000) as f64) as f64;
            }
            if (crossOver && cond1) {
                acc.buy_close(code, 10.0, bar.datetime.as_ref(), bar.open);
            }
            if (bar.high < stopLine) {
                acc.buy_close(code, 10.0, bar.datetime.as_ref(), bar.open);
            }
        }


        lastbar = bar;
    }
    println!("{:?}", acc.history_table());

    //qaaccount::QA_Account::history_table(&mut acc);
}


fn main(){
    let sw = Stopwatch::start_new();
    backtest();
    //let file = File::open("data15.csv").unwrap();
    println!("It took {0:.8} ms",sw.elapsed_ms());
}