//! 画像ファイル名と説明文と撮影場所を記録したJSONファイルを読み込み、データを生成する

use anyhow::Result;
use exif::{DateTime, In, Tag, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{fs::File, io::BufReader, str};

/// 書きだすためのデータ
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct PhotoData {
  /// ファイル名（編集不可にしたい）
  pub file_name: String,
  /// 一意に定まるID
  pub photo_id: String,
  /// 表示するときに使用されるpath
  /// staticフォルダ内の画像を参照するため、
  /// `images/normal/`から始まる
  pub photo_src: String,
  /// 遅延表示するときに使用されるpath
  /// staticフォルダ内の画像を参照するため、
  /// `images/lazy/`から始まる
  pub photo_lazy_src: String,
  /// 画像の説明
  pub alt: String,
  /// 画像タイトル
  pub title: Option<String>,
  /// 撮影日時
  pub year: Option<String>,
  /// 撮影日時
  pub month: Option<String>,
  /// 撮影日時
  pub day: Option<String>,
  /// 撮影時刻（時）
  pub hour: Option<String>,
  /// 撮影時刻（分）
  pub minutes: Option<String>,
  /// 使用ボディ
  pub body: Option<String>,
  /// 使用レンズ
  pub lens: Option<String>,
  /// シャッタースピード
  pub time: Option<String>,
  /// 焦点距離
  pub focal_length: Option<String>,
  /// F値
  #[serde(rename = "F_value")]
  pub f_value: Option<String>,
  /// ISO感度
  pub iso: Option<String>,
  /// 撮影場所
  pub location: String,
}

/// GUIで使う用のデータ
/// データ書き換え対応のためにOption<String>が使えないので
/// 代わりに全てString型で保持する
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GUIPhotoData {
  pub file_name: String,
  pub photo_id: String,
  pub photo_src: String,
  pub photo_lazy_src: String,
  pub alt: String,
  pub title: String,
  pub year: String,
  pub month: String,
  pub day: String,
  pub hour: String,
  pub minutes: String,
  pub body: String,
  pub lens: String,
  pub time: String,
  pub focal_length: String,
  pub f_value: String,
  pub iso: String,
  pub location: String,
}

pub fn gui_photo_data_to_photo_data(gui_photo_data: GUIPhotoData) -> PhotoData {
  PhotoData {
    file_name: gui_photo_data.file_name,
    photo_id: gui_photo_data.photo_id,
    photo_src: gui_photo_data.photo_src,
    photo_lazy_src: gui_photo_data.photo_lazy_src,
    alt: gui_photo_data.alt,
    title: if gui_photo_data.title.is_empty() {
      None
    } else {
      Some(gui_photo_data.title)
    },
    year: if gui_photo_data.year.is_empty() {
      None
    } else {
      Some(gui_photo_data.year)
    },
    month: if gui_photo_data.month.is_empty() {
      None
    } else {
      Some(gui_photo_data.month)
    },
    day: if gui_photo_data.day.is_empty() {
      None
    } else {
      Some(gui_photo_data.day)
    },
    hour: if gui_photo_data.hour.is_empty() {
      None
    } else {
      Some(gui_photo_data.hour)
    },
    minutes: if gui_photo_data.minutes.is_empty() {
      None
    } else {
      Some(gui_photo_data.minutes)
    },
    body: if gui_photo_data.body.is_empty() {
      None
    } else {
      Some(gui_photo_data.body)
    },
    lens: if gui_photo_data.lens.is_empty() {
      None
    } else {
      Some(gui_photo_data.lens)
    },
    time: if gui_photo_data.time.is_empty() {
      None
    } else {
      Some(gui_photo_data.time)
    },
    focal_length: if gui_photo_data.focal_length.is_empty() {
      None
    } else {
      Some(gui_photo_data.focal_length)
    },
    f_value: if gui_photo_data.f_value.is_empty() {
      None
    } else {
      Some(gui_photo_data.f_value)
    },
    iso: if gui_photo_data.iso.is_empty() {
      None
    } else {
      Some(gui_photo_data.iso)
    },
    location: gui_photo_data.location,
  }
}

pub fn photo_data_to_gui_photo_data(photo_data: PhotoData) -> GUIPhotoData {
  GUIPhotoData {
    file_name: photo_data.file_name,
    photo_id: photo_data.photo_id,
    photo_src: photo_data.photo_src,
    photo_lazy_src: photo_data.photo_lazy_src,
    alt: photo_data.alt,
    title: photo_data.title.unwrap_or_default(),
    year: photo_data.year.unwrap_or_default(),
    month: photo_data.month.unwrap_or_default(),
    day: photo_data.day.unwrap_or_default(),
    hour: photo_data.hour.unwrap_or_default(),
    minutes: photo_data.minutes.unwrap_or_default(),
    body: photo_data.body.unwrap_or_default(),
    lens: photo_data.lens.unwrap_or_default(),
    time: photo_data.time.unwrap_or_default(),
    focal_length: photo_data.focal_length.unwrap_or_default(),
    f_value: photo_data.f_value.unwrap_or_default(),
    iso: photo_data.iso.unwrap_or_default(),
    location: photo_data.location,
  }
}

/// 読み込むときのデータ
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ImportPhotoData {
  /// 書きだした画像のpath
  /// originalフォルダ内にあるはず
  pub file_name: String,
  /// 一意性のあるIDを付けてもらう
  /// URLなどにも使われることになる
  pub id: String,
  /// 画像の説明
  pub alt: String,
  /// 撮影場所
  pub location: String,
}

/// 作成したデータを元のJSONファイルに還元するための変換
pub fn gui_photo_data_to_import_photo_data(photo_data: GUIPhotoData) -> ImportPhotoData {
  ImportPhotoData {
    file_name: photo_data.file_name,
    id: photo_data.photo_id,
    alt: photo_data.alt,
    location: photo_data.location,
  }
}

/// jsonファイルのpathからデータを構築する
pub fn load_import_json_file(file_path: &str) -> Result<Vec<ImportPhotoData>> {
  let reader = BufReader::new(File::open(file_path)?);
  let data: Vec<ImportPhotoData> = serde_json::from_reader(reader)?;
  Ok(data)
}

/// photo_data.jsonが保存されているディレクトリのpathから中身を読み取る
pub fn load_photo_data_opt(work_directory: &str) -> HashMap<String, PhotoData> {
  let mut hashmap = HashMap::new();
  let file_path = format!("{}/photo_data.json", work_directory);
  match File::open(file_path) {
    Ok(file) => {
      let reader = BufReader::new(file);
      let data_lst: Vec<PhotoData> = serde_json::from_reader(reader).unwrap();
      for data in data_lst.iter() {
        hashmap.insert(data.clone().photo_id, data.clone());
      }
      hashmap
    }
    Err(_) => hashmap,
  }
}

/// 事前に生成されていたphoto_data.jsonを元にした`PhotoData`と
/// 現像時に手動で作成した元の画像ファイル名などが入る`ImportPhotoData`と
/// 元画像が置かれたフォルダへのpathを受け取って、
/// その中身をもとにJPEGファイルを検索してデータを取り出し、`PhotoData`に変換する
pub fn merge_photo_data_based_and_import_photo_data(
  original_photo_data_lst: &HashMap<String, PhotoData>,
  import_photo_data_lst: &[ImportPhotoData],
  original_path: &str,
) -> Result<(Vec<String>, Vec<PhotoData>)> {
  let mut photo_id_lst = Vec::new();
  let mut photo_data_lst = Vec::new();
  for import_photo_data in import_photo_data_lst.iter() {
    photo_id_lst.push(import_photo_data.clone().id);

    photo_data_lst.push(match original_photo_data_lst.get(&import_photo_data.id) {
      // 既に元のデータがある場合はそちらを優先する
      Some(photo_data) => PhotoData {
        file_name: import_photo_data.file_name.clone(),
        photo_id: import_photo_data.id.clone(),
        photo_src: format!("/images/normal/{}.JPG", import_photo_data.id),
        photo_lazy_src: format!("/images/lazy/{}.JPG", import_photo_data.id),
        alt: import_photo_data.alt.clone(),
        location: import_photo_data.location.clone(),
        ..photo_data.clone()
      },
      // まだデータが無い場合はExifファイルの中身を元に構築する
      None => match parse_exif_data(&format!(
        "{}/{}",
        original_path,
        import_photo_data.clone().file_name
      )) {
        Ok(minimal_exif_data) => PhotoData {
          file_name: import_photo_data.file_name.clone(),
          photo_id: import_photo_data.id.clone(),
          photo_src: format!("/images/normal/{}.JPG", import_photo_data.id),
          photo_lazy_src: format!("/images/lazy/{}.JPG", import_photo_data.id),
          alt: import_photo_data.alt.clone(),
          title: None,
          year: minimal_exif_data.year,
          month: minimal_exif_data.month,
          day: minimal_exif_data.day,
          hour: minimal_exif_data.hour,
          minutes: minimal_exif_data.minutes,
          body: minimal_exif_data.body,
          lens: minimal_exif_data.lens,
          time: minimal_exif_data.time,
          focal_length: minimal_exif_data.focal_length,
          f_value: minimal_exif_data.f_value,
          iso: minimal_exif_data.iso,
          location: import_photo_data.location.clone(),
        },
        Err(_) => PhotoData {
          file_name: import_photo_data.file_name.clone(),
          photo_id: String::new(),
          photo_src: format!("/images/normal/{}.JPG", import_photo_data.id),
          photo_lazy_src: format!("/images/lazy/{}.JPG", import_photo_data.id),
          alt: import_photo_data.alt.clone(),
          title: None,
          year: None,
          month: None,
          day: None,
          hour: None,
          minutes: None,
          body: None,
          lens: None,
          time: None,
          focal_length: None,
          f_value: None,
          iso: None,
          location: import_photo_data.location.clone(),
        },
      },
    })
  }
  Ok((photo_id_lst, photo_data_lst))
}

/// 事前に生成されていた`GUIPhotoData`と`GUIGroupData`と、
/// 現像時に手動で作成した元の画像ファイル名などが入る`ImportPhotoData`と、
/// 元画像が置かれたフォルダへのpathを受け取って、
/// その中身を良い感じに合成して`ImportPhotoData`の中身を反映する
pub fn merge_gui_photo_data_based_and_import_photo_data(
  gui_photo_data_lst: &mut HashMap<String, GUIPhotoData>,
  gui_group_data_lst: &mut HashMap<String, GUIGroupData>,
  import_photo_data_lst: &[ImportPhotoData],
  original_path: &str,
) -> (HashMap<String, GUIPhotoData>, HashMap<String, GUIGroupData>) {
  // photo_dataの更新
  for import_photo_data in import_photo_data_lst.iter() {
    let gui_photo_data_opt = gui_photo_data_lst.get(&import_photo_data.id);
    let data = match gui_photo_data_opt {
      Some(gui_photo_data) => {
        // 良い感じに反映させる
        GUIPhotoData {
          file_name: import_photo_data.file_name.clone(),
          photo_id: import_photo_data.id.clone(),
          photo_src: format!("/images/normal/{}.JPG", import_photo_data.id),
          photo_lazy_src: format!("/images/lazy/{}.JPG", import_photo_data.id),
          alt: import_photo_data.alt.clone(),
          location: import_photo_data.location.clone(),
          ..gui_photo_data.clone()
        }
      }
      None => {
        // 新規データ
        match parse_exif_data(&format!(
          "{}/{}",
          original_path,
          import_photo_data.clone().file_name
        )) {
          Ok(minimal_exif_data) => GUIPhotoData {
            file_name: import_photo_data.file_name.clone(),
            photo_id: import_photo_data.id.clone(),
            photo_src: format!("/images/normal/{}.JPG", import_photo_data.id),
            photo_lazy_src: format!("/images/lazy/{}.JPG", import_photo_data.id),
            alt: import_photo_data.alt.clone(),
            title: String::default(),
            year: minimal_exif_data.year.unwrap_or_default(),
            month: minimal_exif_data.month.unwrap_or_default(),
            day: minimal_exif_data.day.unwrap_or_default(),
            hour: minimal_exif_data.hour.unwrap_or_default(),
            minutes: minimal_exif_data.minutes.unwrap_or_default(),
            body: minimal_exif_data.body.unwrap_or_default(),
            lens: minimal_exif_data.lens.unwrap_or_default(),
            time: minimal_exif_data.time.unwrap_or_default(),
            focal_length: minimal_exif_data.focal_length.unwrap_or_default(),
            f_value: minimal_exif_data.f_value.unwrap_or_default(),
            iso: minimal_exif_data.iso.unwrap_or_default(),
            location: import_photo_data.location.clone(),
          },
          Err(_) => GUIPhotoData {
            file_name: import_photo_data.file_name.clone(),
            photo_id: String::new(),
            photo_src: format!("/images/normal/{}.JPG", import_photo_data.id),
            photo_lazy_src: format!("/images/lazy/{}.JPG", import_photo_data.id),
            alt: import_photo_data.alt.clone(),
            title: String::default(),
            year: String::default(),
            month: String::default(),
            day: String::default(),
            hour: String::default(),
            minutes: String::default(),
            body: String::default(),
            lens: String::default(),
            time: String::default(),
            focal_length: String::default(),
            f_value: String::default(),
            iso: String::default(),
            location: import_photo_data.location.clone(),
          },
        }
      }
    };
    gui_photo_data_lst.insert(import_photo_data.id.to_string(), data);
  }
  // group_dataの更新
  // photo_id_listの中身を検索してIDが存在しているかを確認する
  // IDが無くなっていれば削除する
  let mut new_gui_group_data_lst = HashMap::new();
  for (id, gui_group_data) in gui_group_data_lst.clone().iter() {
    let photo_id_list = gui_group_data
      .photo_id_list
      .iter()
      .filter(|id| gui_photo_data_lst.get(*id).is_some())
      .cloned()
      .collect::<Vec<String>>();
    new_gui_group_data_lst.insert(
      id.to_string(),
      GUIGroupData {
        photo_id_list,
        ..gui_group_data.clone()
      },
    );
  }
  (gui_photo_data_lst.clone(), new_gui_group_data_lst)
}

/// Exifデータの中で必要なもの
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimalExif {
  year: Option<String>,
  month: Option<String>,
  day: Option<String>,
  hour: Option<String>,
  minutes: Option<String>,
  body: Option<String>,
  lens: Option<String>,
  time: Option<String>,
  focal_length: Option<String>,
  f_value: Option<String>,
  iso: Option<String>,
}

/// Exifファイルを解析して必要なデータを取り出す
/// 参照：[https://docs.rs/kamadak-exif/latest/exif/struct.Tag.html#impl-1](https://docs.rs/kamadak-exif/latest/exif/struct.Tag.html#impl-1)
/// 参照：[Exifタグの名称と意味](https://www.vieas.com/exif23.html)
pub fn parse_exif_data(path: &str) -> Result<MinimalExif> {
  let file = File::open(path)?;
  let mut bufreader = BufReader::new(&file);
  let exifreader = exif::Reader::new();
  let exif = exifreader.read_from_container(&mut bufreader)?;
  let (year, month, day, hour, minutes) = match exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
    Some(field) => match field.value {
      Value::Ascii(ref vec) if !vec.is_empty() => {
        let dt = DateTime::from_ascii(&vec[0])?;
        (
          Some(dt.year.to_string()),
          Some(dt.month.to_string()),
          Some(dt.hour.to_string()),
          Some(dt.hour.to_string()),
          Some(dt.minute.to_string()),
        )
      }
      _ => (None, None, None, None, None),
    },
    None => (None, None, None, None, None),
  };
  // レンズのデータ
  let lens_maker = exif
    .get_field(Tag::LensMake, In::PRIMARY)
    .map(|field| field.value.display_as(Tag::LensMake).to_string());
  let lens_model = exif
    .get_field(Tag::LensModel, In::PRIMARY)
    .map(|field| field.value.display_as(Tag::LensModel).to_string());
  let lens = match (lens_maker, lens_model) {
    (Some(maker), Some(model)) => Some(format!("{maker} {model}")),
    (None, Some(model)) => Some(model),
    _ => None,
  };
  // シャッタースピード
  let time = exif
    .get_field(Tag::ShutterSpeedValue, In::PRIMARY)
    .map(|field| field.value.display_as(Tag::ShutterSpeedValue).to_string());
  // 焦点距離
  let focal_length = exif
    .get_field(Tag::FocalLength, In::PRIMARY)
    .map(|field| field.value.display_as(Tag::FocalLength).to_string());
  // F値
  let f_value = exif
    .get_field(Tag::FNumber, In::PRIMARY)
    .map(|field| field.value.display_as(Tag::FNumber).to_string());
  // ISO感度
  let iso = exif
    .get_field(Tag::ISOSpeed, In::PRIMARY)
    .map(|field| field.value.display_as(Tag::ISOSpeed).to_string());
  let v = MinimalExif {
    year,
    month,
    day,
    hour,
    minutes,
    body: None,
    lens,
    time,
    focal_length,
    f_value,
    iso,
  };
  Ok(v)
}

/// 出力する`group_data.json`ファイルに書き出す内容
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupData {
  pub group_id: String,
  pub photo_id_list: Vec<String>,
  pub year: Option<String>,
  pub month: Option<String>,
  pub day: Option<String>,
  pub hour: Option<String>,
  pub minutes: Option<String>,
  pub title: String,
  pub description: String,
  pub location: Option<String>,
}

/// `GroupData`をGUIで扱うためのデータ構造
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GUIGroupData {
  pub group_id: String,
  pub photo_id_list: Vec<String>,
  pub year: String,
  pub month: String,
  pub day: String,
  pub hour: String,
  pub minutes: String,
  pub title: String,
  pub description: String,
  pub location: String,
}

pub fn make_dummy_gui_group_data() -> GUIGroupData {
  GUIGroupData {
    group_id: String::new(),
    photo_id_list: Vec::new(),
    year: String::new(),
    month: String::new(),
    day: String::new(),
    hour: String::new(),
    minutes: String::new(),
    title: String::new(),
    description: String::new(),
    location: String::new(),
  }
}

pub fn gui_group_data_to_group_data(gui_group_data: GUIGroupData) -> GroupData {
  GroupData {
    group_id: gui_group_data.group_id,
    photo_id_list: gui_group_data.photo_id_list,
    year: if gui_group_data.year.is_empty() {
      None
    } else {
      Some(gui_group_data.year)
    },
    month: if gui_group_data.month.is_empty() {
      None
    } else {
      Some(gui_group_data.month)
    },
    day: if gui_group_data.day.is_empty() {
      None
    } else {
      Some(gui_group_data.day)
    },
    hour: if gui_group_data.hour.is_empty() {
      None
    } else {
      Some(gui_group_data.hour)
    },
    minutes: if gui_group_data.minutes.is_empty() {
      None
    } else {
      Some(gui_group_data.minutes)
    },
    title: gui_group_data.title,
    description: gui_group_data.description,
    location: if gui_group_data.location.is_empty() {
      None
    } else {
      Some(gui_group_data.location)
    },
  }
}

pub fn group_data_to_gui_group_data(group_data: GroupData) -> GUIGroupData {
  GUIGroupData {
    group_id: group_data.group_id,
    photo_id_list: group_data.photo_id_list,
    year: group_data.year.unwrap_or_default(),
    month: group_data.month.unwrap_or_default(),
    day: group_data.day.unwrap_or_default(),
    hour: group_data.hour.unwrap_or_default(),
    minutes: group_data.minutes.unwrap_or_default(),
    title: group_data.title,
    description: group_data.description,
    location: group_data.location.unwrap_or_default(),
  }
}

/// jsonファイルのpathからデータを構築する
pub fn load_group_data_from_work_directory(work_directory: &str) -> Result<Vec<GroupData>> {
  let file_path = format!("{}/group_data.json", work_directory);
  match File::open(file_path) {
    Ok(file) => {
      let reader = BufReader::new(file);
      let data: Vec<GroupData> = serde_json::from_reader(reader)?;
      Ok(data)
    }
    Err(_) => Ok(Vec::new()),
  }
}
