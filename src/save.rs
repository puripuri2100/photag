//! ファイルの保存に関する制御をする
//! データファイルの書き出し・画像ファイルの書き出しの他、適度なタイミングでのデータの読み込みとそれの反映も制御する

use anyhow::Result;
use chrono::{DateTime, FixedOffset, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::{fs::File, io::BufReader, io::Write};

const MINUTES: i32 = 60;
/// 画像を保存する間隔
pub const SAVE_IMAGE_DIFF_TIME: i32 = MINUTES * 7;
/// JSONファイルを保存する間隔
pub const SAVE_JSON_DIFF_TIME: i32 = MINUTES;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimeInfo {
  id: String,
  time: DateTime<FixedOffset>,
}

/// 画像ファイルのpathからタイムスタンプを取得する
pub fn get_file_timestamp(path: &str) -> Option<DateTime<FixedOffset>> {
  let metadata_res = fs::metadata(path);
  match metadata_res {
    Ok(metadata) => {
      let timestamp_res = metadata.created();
      match timestamp_res {
        Ok(time) => {
          let datetime_local = DateTime::<Local>::from(time);
          let datetime = datetime_local.with_timezone(datetime_local.offset());
          Some(datetime)
        }
        Err(_) => None,
      }
    }
    Err(_) => None,
  }
}

/// 外部に保存した「各画像の変換時刻」の情報を取得する
pub fn get_time_info_lst(work_dir: &str) -> HashMap<String, DateTime<FixedOffset>> {
  let file_path = format!("{}/time.json", work_dir);
  let mut data = HashMap::new();
  let file_res = File::open(file_path);
  match file_res {
    Ok(file) => {
      let reader = BufReader::new(file);
      let time_info_lst: Vec<TimeInfo> = serde_json::from_reader(reader).unwrap();
      for time_info in time_info_lst {
        data.insert(time_info.id, time_info.time);
      }
      data
    }
    Err(_) => data,
  }
}

pub fn save_time_info_lst(
  work_dir: &str,
  time_info_lst: &HashMap<String, DateTime<FixedOffset>>,
) -> Result<()> {
  let path = format!("{}/time.json", work_dir);
  let mut file = File::create(path)?;
  let mut v = Vec::new();
  for (id, time) in time_info_lst.iter() {
    v.push(TimeInfo {
      id: id.clone(),
      time: *time,
    })
  }
  let json_str = serde_json::to_string_pretty(&v)?;
  let buf = json_str.into_bytes();
  file.write_all(&buf)?;
  file.flush()?;
  Ok(())
}

pub fn get_now() -> DateTime<FixedOffset> {
  let now = Local::now();
  now.with_timezone(now.offset())
}

pub fn time_add_sec(time: DateTime<FixedOffset>, sec: i32) -> DateTime<FixedOffset> {
  let datetime = FixedOffset::east(sec);
  time + datetime
}
