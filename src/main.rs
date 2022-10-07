extern crate core;

use std::{env, io};
use std::ops::Sub;

use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};

fn main() {
    let args = parse_args();
    alfred_time_workflow(args)
}

fn alfred_time_workflow(args: Args) {
    let time = convert_to_date_time(&args.query);
    alfred_time_workflow_output(&time)
}

#[derive(Debug, Clone)]
struct Args {
    query: String,
}

fn parse_args() -> Args {
    let args: Vec<String> = env::args().collect();
    let mut query: String = String::from("");
    for (index, val) in args.iter().enumerate() {
        let next = index + 1;
        match val.as_str() {
            "-q" => query = args[next].to_string(),
            _ => {}
        };
    }
    Args { query }
}

fn is_number(str: &String) -> bool {
    let mut flag = true;
    for x in str.chars() {
        if !x.is_ascii_digit() {
            flag = false;
        }
    }
    flag
}

fn convert_to_date_time(str: &String) -> DateTime<Utc> {
    if str.is_empty() {
        return DateTime::default();
    };
    let result: DateTime<Utc>;
    if is_number(str) {
        let len = str.len();
        result = match len {
            9 => sec_ts_to_date_time(str),
            13 => ms_ts_to_date_time(str),
            _ => sec_ts_to_date_time(str)
        };
    } else {
        result = str_to_date_time(str);
    };
    result
}

fn str_to_date_time(str: &String) -> DateTime<Utc> {
    let  fmt= String::from("%Y-%m-%d %H:%M:%S %z");
    let f = DateTime::parse_from_str;
    let mut date_str = String::from(str);
    if is_date(str) {
        date_str.push_str(" 00:00:00");
    }
    date_str.push_str(" +0000");
    let mut  time = match f(&date_str, &*fmt) {
        Ok(datetime) => datetime,
        Err(err) => {
            println!("Parse failed! err: {}", err);
            DateTime::default()
        }
    };
    let offset = FixedOffset::east(8 * 3600);
    time = time.sub(offset);
    DateTime::from(time)
}

fn is_date(str: &String) -> bool {
    str.contains("-") && !str.contains(":")
}

fn sec_ts_to_date_time(str: &String) -> DateTime<Utc> {
    let secs: &i64 = &str.parse::<i64>().unwrap();
    let time = NaiveDateTime::from_timestamp(*secs, 0);
    native_to_date_time(&time)
}

fn ms_ts_to_date_time(str: &String) -> DateTime<Utc> {
    let secs: &i64 = &str[0..10].parse::<i64>().unwrap();
    let nsecs: &u32 = &str[11..13].parse::<u32>().unwrap();
    let time = NaiveDateTime::from_timestamp(*secs, *nsecs);
    native_to_date_time(&time)
}

fn native_to_date_time(datetime: &NaiveDateTime) -> DateTime<Utc> {
    DateTime::from_utc(*datetime, Utc)
}


fn local_date_time(datetime: &DateTime<Utc>) -> DateTime<FixedOffset> {
    let offset = FixedOffset::east(8 * 3600);
    datetime.with_timezone(&offset)
}

#[allow(dead_code)]
fn sum(a: i32, b: i32) -> i32 {
    a + b
}

const OUTPUT_FMT: &str = "%Y-%m-%d %H:%M:%S";

fn alfred_time_workflow_output(time: &DateTime<Utc>) {
    let local_time: DateTime<FixedOffset> = local_date_time(&time);
    let outputs: Vec<Output> = vec![
        Output {
            title: "时间戳(毫秒)".to_string(),
            value: time.timestamp_millis().to_string(),
        },
        Output {
            title: "UTC+8".to_string(),
            value: local_time.format(OUTPUT_FMT).to_string(),
        },
        Output {
            title: "UTC".to_string(),
            value: time.format(OUTPUT_FMT).to_string(),
        },
    ];
    let items: Vec<alfred::Item> = outputs
        .into_iter()
        .map(|x| {
            alfred::ItemBuilder::new(x.value.clone())
                .subtitle(x.title.clone())
                .arg(x.value.clone())
                .quicklook_url(x.value.clone())
                .text_copy(x.value.clone())
                .icon_filetype("fileicon")
                .icon_file("../resource/icon.png")
                .into_item()
        })
        .collect();
    alfred::json::Builder::with_items(&items)
        .write(io::stdout())
        .expect("Couldn't write items to Alfred!")
}

#[derive(Clone, Debug)]
pub struct Output {
    pub title: String,
    pub value: String,
}

#[cfg(test)]
mod tests {
    use chrono::format::Fixed::TimezoneOffset;
    use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, Utc};

    use crate::{alfred_time_workflow, convert_to_date_time, ms_ts_to_date_time, str_to_date_time, sum, Args, OUTPUT_FMT, native_to_date_time};

    const DATE_TIME_STR: &str = "2022-10-04 13:24:54";
    const DATE_STR: &str = "2022-10-04";
    const MS_STR: &str = "1664861094000";
    const SEC_STR: &str = "1664861094";

    #[test]
    fn sum_test() {
        assert_eq!(sum(1, 2), 3);
    }

    #[test]
    fn test() {
        let date_time = convert_to_date_time(&String::from(DATE_TIME_STR));
        let date_time_by_ms = convert_to_date_time(&String::from(MS_STR));
        let date_time_by_sec = convert_to_date_time(&String::from(SEC_STR));
        let date = convert_to_date_time(&String::from(DATE_STR));
        println!("date: {}", date);
        println!("0: {}, 1: {}, 2:{}", date_time.timestamp(), date_time_by_ms.timestamp(), date_time_by_sec.timestamp());
        assert_eq!(date_time, date_time_by_ms);
        assert_eq!(date_time, date_time_by_sec);
        println!("date_time: {}", date_time);
        let date0 = convert_to_date_time(&String::from("1000"));
        println!("date0: {}", date0);
    }

    #[test]
    fn test_alfred() {
        alfred_time_workflow(Args {
            query: String::from(DATE_TIME_STR),
        });
        alfred_time_workflow(Args {
            query: String::from(MS_STR),
        });
        alfred_time_workflow(Args {
            query: String::from(SEC_STR),
        });
    }

    #[test]
    fn ms_ts_to_date_time_test() {
        // 2022-10-04 13:24:54 1664861094 1664861094_000
        let ts = native_to_date_time(&NaiveDateTime::from_timestamp(1664861094, 0));
        println!("{}", ts);
        let ms_c_time = ms_ts_to_date_time(&String::from(MS_STR));
        println!("{}", ms_c_time);
        assert_eq!(ts, ms_c_time);
    }

    #[test]
    fn str_to_date_time_test() {
        // 2022-10-04 13:24:54
        let datetime_str_0 = String::from("2022-10-04 13:24:54");
        let d0 = str_to_date_time(&datetime_str_0);
        println!("d0: {}", d0);
        let datetime_str_1 = String::from("2022-10-04");
        let d1 = str_to_date_time(&datetime_str_1);
        println!("d1: {}", d1);
    }

    #[test]
    fn parse_from_str_test() {
        let parse_from_str = NaiveDateTime::parse_from_str;
        let fmt = String::from("%Y-%m-%d %H:%M:%S");
        let mut datetime = parse_from_str("2015-09-05 23:56:04", &*fmt);
        assert_eq!(
            datetime,
            Ok(NaiveDate::from_ymd(2015, 9, 5).and_hms(23, 56, 4))
        );
        assert_eq!(
            parse_from_str("5sep2015pm012345.6789", "%d%b%Y%p%I%M%S%.f"),
            Ok(NaiveDate::from_ymd(2015, 9, 5).and_hms_micro(13, 23, 45, 678_900))
        );
        datetime = parse_from_str("2015-09-05 23:56:04", "%Y-%m-%d %H:%M:%S+8");
        //print!("datetime: {}, ts: {}", datetime.unwrap(), datetime.unwrap().timestamp());
    }

    #[test]
    fn date_time_parse_from_str_test() {
        let mut str = String::from(DATE_TIME_STR);
        str.push_str(" +0000");
        println!("str: {}", str);
        let time = DateTime::parse_from_str(&str, "%Y-%m-%d %H:%M:%S %z");
        match time {
            Ok(x) => println!("time: {}", x),
            Err(err) => println!("err: {}", err)
        };
        str = String::from("2022-10-10");
        str.push_str(" 00:00:00");
        str.push_str(" +0000");
        match  DateTime::parse_from_str(&str, "%Y-%m-%d %H:%M:%S %z"){
            Ok(x) => print!("success: {}", x),
            Err(err) => println!("err: {}", err)
        }
    }
}
