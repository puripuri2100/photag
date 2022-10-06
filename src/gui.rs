use chrono::{DateTime, FixedOffset};
use eframe::{
  egui,
  egui::{FontData, FontDefinitions, FontFamily},
};
use egui_extras::RetainedImage;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use crate::image;
use crate::photodata::{self, GUIGroupData, GUIPhotoData};
use crate::save;

#[derive(Clone, Debug)]
pub struct PhotagApp {
  /// 現在のメイン画面に表示するものを決めるためにモードを保持したい
  /// - 写真データの編集モード
  /// - 写真グループの編集モード
  pub mode: Mode,
  /// 画像IDのリスト
  pub photo_id_lst: Vec<String>,
  /// idと画像のデータのペア
  pub gui_photo_data_lst: HashMap<String, photodata::GUIPhotoData>,
  /// グループIDのリスト
  pub group_id_lst: Vec<String>,
  /// idとグループのデータのペア
  pub gui_group_data_lst: HashMap<String, photodata::GUIGroupData>,
  /// idと現像後の画像への絶対pathのペアを保持する
  pub thumbnail_lst: HashMap<String, Vec<u8>>,
  /// 現像時に手で作ったJSONファイルへのpath
  pub input_json_path: String,
  /// オリジナル画像が入っているフォルダへのpath
  pub original_image_folder_path: String,
  /// 画質を落とした画像を出力したり、グループや写真のデータのJSONファイルを出力したり
  /// するためのフォルダへのpath
  pub work_directory_path: String,
  /// 今現在編集しようとしているIDの情報を格納する（画像・グループ共通）
  pub now_id: String,
  /// 新規作成するときのためのダミーのグループデータ
  pub dummy_group_data: photodata::GUIGroupData,
  /// 画像を保存した時刻を保持する
  pub image_save_time_lst: HashMap<String, DateTime<FixedOffset>>,
  /// JSONファイル等を書き出した時刻を保持する
  pub json_save_time: DateTime<FixedOffset>,
  /// 画像を書き出した時刻を保持する
  pub image_save_time: DateTime<FixedOffset>,
}

/// メイン画面に表示するものを決めるためのモード情報
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mode {
  /// 写真データの編集モード
  EditPhotoData,
  /// 写真グループの編集モード
  EditGroupData,
}

fn setup_japanese_fonts(ctx: &egui::Context) {
  let mut fonts = FontDefinitions::default();
  fonts.font_data.insert(
    "ipaexg".to_owned(),
    FontData::from_static(include_bytes!(
      "./../assets/fonts/IPAexfont00401/ipaexg.ttf"
    )),
  );
  fonts
    .families
    .get_mut(&FontFamily::Proportional)
    .unwrap()
    .insert(0, "ipaexg".to_owned());
  fonts
    .families
    .get_mut(&FontFamily::Monospace)
    .unwrap()
    .push("ipaexg".to_owned());
  ctx.set_fonts(fonts);
}

impl PhotagApp {
  pub fn new(
    cc: &eframe::CreationContext<'_>,
    input_json_path: String,
    original_image_folder_path: String,
    work_directory_path: String,
  ) -> Self {
    setup_japanese_fonts(&cc.egui_ctx);
    let import_photo_data_lst = photodata::load_import_json_file(&input_json_path).unwrap();
    let photo_data_opt = photodata::load_photo_data_opt(&work_directory_path);
    let (photo_id_lst, photo_data_lst) = photodata::merge_photo_data_based_and_import_photo_data(
      &photo_data_opt,
      &import_photo_data_lst,
      &original_image_folder_path,
    )
    .unwrap();
    let mut gui_photo_data_lst = HashMap::new();
    for photo_data in photo_data_lst.iter() {
      gui_photo_data_lst.insert(
        photo_data.photo_id.clone(),
        photodata::photo_data_to_gui_photo_data(photo_data.clone()),
      );
    }
    let group_data_lst =
      photodata::load_group_data_from_work_directory(&work_directory_path).unwrap();
    let mut group_id_lst = Vec::new();
    let mut gui_group_data_lst = HashMap::new();
    for group_data in group_data_lst.iter() {
      group_id_lst.push(group_data.clone().group_id);
      gui_group_data_lst.insert(
        group_data.clone().group_id,
        photodata::group_data_to_gui_group_data(group_data.clone()),
      );
    }
    let mut time_info_lst = save::get_time_info_lst(&work_directory_path);
    let mut thumbnail_lst = HashMap::new();
    for import_photo_data in import_photo_data_lst.iter() {
      // 画像ファイルは重いので、アクセスする階数をできるだけ減らしたい
      let image_path = format!(
        "{}/{}",
        original_image_folder_path, import_photo_data.file_name
      );
      // ファイルのバイナリデータを取り出す
      let raw_data = image::open_file(&image_path).unwrap();
      // 起動時に処理する画像は固定されているため、
      // このタイミングで画像を圧縮して保存すれば
      // 次の起動まで何もしなくて良い
      if let Some(time) = time_info_lst.get(&import_photo_data.id) {
        // 書き出し時刻がある場合の処理
        let time_stamp = save::get_file_timestamp(&image_path);
        match time_stamp {
          Some(time_stamp) => {
            if time < &time_stamp {
              // 画像のタイムスタンプの方が遅いため、新規画像と判定して書き出し処理を行う
              save_image_compression_lazy(
                &raw_data,
                &format!(
                  "{}/images/lazy/{}.JPG",
                  work_directory_path, import_photo_data.id
                ),
              );
              save_image_compression_normal(
                &raw_data,
                &format!(
                  "{}/images/normal/{}.JPG",
                  work_directory_path, import_photo_data.id
                ),
              );
              let now = save::get_now();
              time_info_lst.insert(import_photo_data.id.to_string(), now);
            }
          }
          None => {
            // タイムスタンプが無いので念のため書き出す
            // 画像のタイムスタンプの方が遅いため、新規画像と判定して書き出し処理を行う
            save_image_compression_lazy(
              &raw_data,
              &format!(
                "{}/images/lazy/{}.JPG",
                work_directory_path, import_photo_data.id
              ),
            );
            save_image_compression_normal(
              &raw_data,
              &format!(
                "{}/images/normal/{}.JPG",
                work_directory_path, import_photo_data.id
              ),
            );
            let now = save::get_now();
            time_info_lst.insert(import_photo_data.id.to_string(), now);
          }
        }
      } else {
        // 書き出し時刻がないため「新規画像」と認定して書き出し処理を行う
        save_image_compression_lazy(
          &raw_data,
          &format!(
            "{}/images/lazy/{}.JPG",
            work_directory_path, import_photo_data.id
          ),
        );
        save_image_compression_normal(
          &raw_data,
          &format!(
            "{}/images/normal/{}.JPG",
            work_directory_path, import_photo_data.id
          ),
        );
        let now = save::get_now();
        time_info_lst.insert(import_photo_data.id.to_string(), now);
      };
      // サムネイル用に圧縮したデータを生成して登録
      thumbnail_lst.insert(
        import_photo_data.id.to_string(),
        image::compression(&raw_data, 70.0, 600).unwrap(),
      );
    }

    let now = save::get_now();

    PhotagApp {
      mode: Mode::EditPhotoData,
      photo_id_lst,
      gui_photo_data_lst,
      group_id_lst,
      gui_group_data_lst,
      thumbnail_lst,
      input_json_path,
      original_image_folder_path,
      work_directory_path,
      now_id: String::new(),
      dummy_group_data: photodata::make_dummy_gui_group_data(),
      image_save_time_lst: time_info_lst,
      json_save_time: now,
      image_save_time: now,
    }
  }
}

impl eframe::App for PhotagApp {
  // 終了時のイベント
  fn on_close_event(&mut self) -> bool {
    let Self {
      photo_id_lst,
      gui_photo_data_lst,
      group_id_lst,
      gui_group_data_lst,
      input_json_path,
      work_directory_path,
      image_save_time_lst,
      ..
    } = self;
    // JSONファイルを保存
    save_file(
      photo_id_lst,
      gui_photo_data_lst,
      group_id_lst,
      gui_group_data_lst,
      input_json_path,
      work_directory_path,
    );
    // ファイルの保存時刻の情報を保存
    save::save_time_info_lst(work_directory_path, image_save_time_lst).unwrap();

    // trueのときはそのまま終了イベントが継続する
    true
  }

  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    let Self {
      mode,
      photo_id_lst,
      gui_photo_data_lst,
      group_id_lst,
      gui_group_data_lst,
      thumbnail_lst,
      now_id,
      input_json_path,
      original_image_folder_path,
      work_directory_path,
      image_save_time_lst,
      json_save_time,
      image_save_time,
      ..
    } = self;

    let now = save::get_now();
    if save::time_add_sec(*json_save_time, save::SAVE_JSON_DIFF_TIME) > now {
      // 一定時間が経過したので、JSONファイルの読み込み等を行って更新が無いかを確認する
      // 更新があった場合、データのアップデートと新規保存を行う
      match save::get_file_timestamp(input_json_path) {
        Some(timestamp) => {
            let import_photo_data_lst = photodata::load_import_json_file(input_json_path).unwrap();
            let (new_gui_photo_data_lst, new_gui_group_data_lst) =
              photodata::merge_gui_photo_data_based_and_import_photo_data(
                gui_photo_data_lst,
                gui_group_data_lst,
                &import_photo_data_lst,
                original_image_folder_path,
              );
            // JSONファイルを保存
            save_file(
              photo_id_lst,
              &new_gui_photo_data_lst,
              group_id_lst,
              &new_gui_group_data_lst,
              input_json_path,
              work_directory_path,
            );
        }
        None => {
          // JSONファイルを保存
          save_file(
            photo_id_lst,
            gui_photo_data_lst,
            group_id_lst,
            gui_group_data_lst,
            input_json_path,
            work_directory_path,
          );
        }
      }
      *json_save_time = save::get_now();
    }

    if save::time_add_sec(*image_save_time, save::SAVE_IMAGE_DIFF_TIME) > now {
      // 一定時間が経過したので、画像ファイルに更新が無いかを確認する
      // 更新があった場合、当該ファイルの書き出し処理も行う
      for (id, gui_photo_data) in gui_photo_data_lst.iter() {
        let image_path = format!(
          "{}/{}",
          original_image_folder_path, gui_photo_data.file_name
        );
        if let Some(time) = image_save_time_lst.get(&gui_photo_data.photo_id) {
          // 書き出し時刻がある場合の処理
          let time_stamp = save::get_file_timestamp(&image_path);
          match time_stamp {
            Some(time_stamp) => {
              if time < &time_stamp {
                // 画像のタイムスタンプの方が遅いため、新規画像と判定して書き出し処理を行う
                let raw_data = thumbnail_lst.get(id).unwrap();
                save_image_compression_lazy(
                  raw_data,
                  &format!("{}/images/lazy/{}.JPG", work_directory_path, id),
                );
                save_image_compression_normal(
                  raw_data,
                  &format!("{}/images/normal/{}.JPG", work_directory_path, id),
                );
                let now = save::get_now();
                image_save_time_lst.insert(id.to_string(), now);
              }
            }
            None => {
              // タイムスタンプが無い・ファイルが無いので念のため書き出す
              // 画像のタイムスタンプの方が遅いため、新規画像と判定して書き出し処理を行う
              let raw_data = thumbnail_lst.get(id).unwrap();
              save_image_compression_lazy(
                raw_data,
                &format!("{}/images/lazy/{}.JPG", work_directory_path, id),
              );
              save_image_compression_normal(
                raw_data,
                &format!("{}/images/normal/{}.JPG", work_directory_path, id),
              );
              let now = save::get_now();
              image_save_time_lst.insert(id.to_string(), now);
            }
          }
        } else {
          // 書き出し時刻がないため「新規画像」と認定して書き出し処理を行う
          let raw_data = image::open_file(&image_path).unwrap();
          save_image_compression_lazy(
            &raw_data,
            &format!("{}/images/lazy/{}.JPG", work_directory_path, id),
          );
          save_image_compression_normal(
            &raw_data,
            &format!("{}/images/normal/{}.JPG", work_directory_path, id),
          );
          thumbnail_lst.insert(
            id.to_string(),
            image::compression(&raw_data, 70.0, 600).unwrap(),
          );
          let now = save::get_now();
          image_save_time_lst.insert(id.to_string(), now);
        };
      }
      // ファイルの保存時刻の情報を保存
      save::save_time_info_lst(work_directory_path, image_save_time_lst).unwrap();
      *image_save_time = save::get_now();
    }

    egui::SidePanel::left("side_panel")
      .min_width(50.0)
      .show(ctx, |ui| match mode {
        Mode::EditPhotoData => {
          ui.heading("画像データ編集ページ");
          let keep_button = ui.button("保存").clicked();
          ui.heading("グループデータ編集ページ");
          let switch_button = ui.button("切り替え").clicked();
          if switch_button {
            *mode = Mode::EditGroupData;
            *now_id = String::new();
          }
          ui.heading("画像ID一覧");
          egui::ScrollArea::vertical().show(ui, |ui| {
            for photo_id in photo_id_lst.iter() {
              let button = if photo_id == now_id {
                egui::Button::new(photo_id).fill(egui::Color32::KHAKI)
              } else {
                egui::Button::new(photo_id)
              };
              if ui.add(button).clicked() {
                *mode = Mode::EditPhotoData;
                *now_id = photo_id.clone();
              }
            }
          });
          if keep_button {
            // JSONファイルを保存
            save_file(
              photo_id_lst,
              gui_photo_data_lst,
              group_id_lst,
              gui_group_data_lst,
              input_json_path,
              work_directory_path,
            );
            // ファイルの保存時刻の情報を保存
            save::save_time_info_lst(work_directory_path, image_save_time_lst).unwrap();
            *json_save_time = save::get_now();
          }
        }
        Mode::EditGroupData => {
          ui.heading("グループデータ作成ページ");
          let keep_button = ui.button("保存").clicked();
          ui.heading("画像データ編集ページ");
          let switch_button = ui.button("切り替え").clicked();
          if switch_button {
            *mode = Mode::EditPhotoData;
            *now_id = String::new();
          }
          ui.heading("グループID一覧");
          egui::ScrollArea::vertical().show(ui, |ui| {
            let new_button = ui.button("新規").clicked();
            if new_button {
              *mode = Mode::EditGroupData;
              *now_id = String::new();
            }
            for group_id in group_id_lst.iter() {
              let button = if group_id == now_id {
                egui::Button::new(group_id).fill(egui::Color32::KHAKI)
              } else {
                egui::Button::new(group_id)
              };
              if ui.add(button).clicked() {
                *mode = Mode::EditGroupData;
                *now_id = group_id.clone();
              }
            }
          });
          if keep_button {
            // JSONファイルを保存
            save_file(
              photo_id_lst,
              gui_photo_data_lst,
              group_id_lst,
              gui_group_data_lst,
              input_json_path,
              work_directory_path,
            );
            // ファイルの保存時刻の情報を保存
            save::save_time_info_lst(work_directory_path, image_save_time_lst).unwrap();
            *json_save_time = save::get_now();
          }
        }
      });

    egui::CentralPanel::default().show(ctx, |ui| {
      let Self {
        mode,
        gui_photo_data_lst,
        group_id_lst,
        gui_group_data_lst,
        now_id,
        dummy_group_data,
        ..
      } = self;
      match mode {
        Mode::EditPhotoData => {
          if !now_id.is_empty() {
            let mut photo_data = gui_photo_data_lst.get(now_id).unwrap().clone();
            ui.heading(format!("{}({})", &now_id, photo_data.file_name));
            ui.vertical(|ui| {
              ui.set_width(300.0);
              ui.horizontal(|ui| {
                ui.label("alt：");
                ui.text_edit_singleline(&mut photo_data.alt);
              });
              ui.horizontal(|ui| {
                ui.label("title：");
                ui.text_edit_singleline(&mut photo_data.title);
              });
              ui.horizontal(|ui| {
                ui.label("撮影場所：");
                ui.text_edit_singleline(&mut photo_data.location);
              });
              ui.horizontal(|ui| {
                ui.label("ISO感度：");
                ui.text_edit_singleline(&mut photo_data.iso);
              });
              ui.horizontal(|ui| {
                ui.label("F値：");
                ui.text_edit_singleline(&mut photo_data.f_value);
              });
              ui.horizontal(|ui| {
                ui.label("シャッタースピード：");
                ui.text_edit_singleline(&mut photo_data.time);
              });
              ui.horizontal(|ui| {
                ui.label("撮影日時：");
                ui.text_edit_singleline(&mut photo_data.year);
                ui.label("/");
                ui.text_edit_singleline(&mut photo_data.month);
                ui.label("/");
                ui.text_edit_singleline(&mut photo_data.day);
                ui.label(", ");
                ui.text_edit_singleline(&mut photo_data.hour);
                ui.label(":");
                ui.text_edit_singleline(&mut photo_data.minutes);
              });
              ui.horizontal(|ui| {
                ui.label("使用機材：");
                ui.text_edit_singleline(&mut photo_data.body);
              });
              ui.horizontal(|ui| {
                ui.label("               + ");
                ui.text_edit_singleline(&mut photo_data.lens);
              });
              ui.horizontal(|ui| {
                ui.label("焦点距離：");
                ui.text_edit_singleline(&mut photo_data.focal_length);
                ui.label("mm");
              });
              // サムネイル生成
              let image_buf = thumbnail_lst.get(now_id).unwrap();
              let image = RetainedImage::from_image_bytes(&*now_id, image_buf).unwrap();
              image.show_size(ui, calculate_image_size(300.0, &image.size()));
            });
            ui.label("グループへの登録");
            let mut group_check_lst =
              make_group_check_lst(now_id, group_id_lst, gui_group_data_lst);
            egui::ScrollArea::vertical().show(ui, |ui| {
              for i in 0..group_check_lst.len() {
                ui.horizontal(|ui| {
                  let group_id = &group_id_lst[i];
                  let group_data = gui_group_data_lst.get(group_id).unwrap();
                  ui.checkbox(&mut group_check_lst[i].1, group_id);
                  ui.label(format!(
                    "  {}（{}）",
                    group_data.title, group_data.description
                  ));
                });
              }
            });
            update_group_data(now_id, &group_check_lst, group_id_lst, gui_group_data_lst);
            gui_photo_data_lst.insert(now_id.clone(), photo_data);
          }
        }
        Mode::EditGroupData => {
          if now_id.is_empty() {
            ui.heading("新規グループ作成");
            let make_button = ui.button("作成").clicked();
            if make_button {
              if dummy_group_data.group_id.is_empty()
                || dummy_group_data.title.is_empty()
                || dummy_group_data.description.is_empty()
              {
                ui.label("必須のデータが入力されていません");
                eprintln!("必須のデータが入力されていないため、グループを新規に作成できません");
              } else {
                group_id_lst.push(dummy_group_data.clone().group_id);
                gui_group_data_lst
                  .insert(dummy_group_data.clone().group_id, dummy_group_data.clone());
                *dummy_group_data = photodata::make_dummy_gui_group_data()
              }
            }
            ui.vertical(|ui| {
              ui.set_width(500.0);
              ui.horizontal(|ui| {
                ui.label("グループID");
                ui.text_edit_singleline(&mut dummy_group_data.group_id);
              });
              ui.horizontal(|ui| {
                ui.label("タイトル（必須）");
                ui.text_edit_singleline(&mut dummy_group_data.title);
              });
              ui.horizontal(|ui| {
                ui.label("説明（必須）");
                ui.text_edit_singleline(&mut dummy_group_data.description);
              });
              ui.horizontal(|ui| {
                ui.label("撮影地点");
                ui.text_edit_singleline(&mut dummy_group_data.location);
              });
              ui.horizontal(|ui| {
                ui.label("撮影年月日");
                ui.text_edit_singleline(&mut dummy_group_data.year);
                ui.label("/");
                ui.text_edit_singleline(&mut dummy_group_data.month);
                ui.label("/");
                ui.text_edit_singleline(&mut dummy_group_data.day);
              });
              ui.horizontal(|ui| {
                ui.label("撮影時刻");
                ui.text_edit_singleline(&mut dummy_group_data.hour);
                ui.label(":");
                ui.text_edit_singleline(&mut dummy_group_data.minutes);
              });
            });
          } else {
            ui.heading(now_id.clone());
            let delete_button = ui.button("削除").clicked();
            if delete_button {
              *group_id_lst = group_id_lst
                .iter()
                .filter(|id| id.to_string() != now_id.clone())
                .cloned()
                .collect();
              *now_id = String::new();
            }
            if !delete_button {
              let mut group_data = gui_group_data_lst.get(now_id).unwrap().clone();
              ui.vertical(|ui| {
                ui.set_width(500.0);
                ui.horizontal(|ui| {
                  ui.label("グループID");
                  ui.text_edit_singleline(&mut group_data.group_id);
                });
                ui.horizontal(|ui| {
                  ui.label("タイトル（必須）");
                  ui.text_edit_singleline(&mut group_data.title);
                });
                ui.horizontal(|ui| {
                  ui.label("説明（必須）");
                  ui.text_edit_singleline(&mut group_data.description);
                });
                ui.horizontal(|ui| {
                  ui.label("撮影地点");
                  ui.text_edit_singleline(&mut group_data.location);
                });
                ui.horizontal(|ui| {
                  ui.label("撮影年月日");
                  ui.text_edit_singleline(&mut group_data.year);
                  ui.label("/");
                  ui.text_edit_singleline(&mut group_data.month);
                  ui.label("/");
                  ui.text_edit_singleline(&mut group_data.day);
                });
                ui.horizontal(|ui| {
                  ui.label("撮影時刻");
                  ui.text_edit_singleline(&mut group_data.hour);
                  ui.label(":");
                  ui.text_edit_singleline(&mut group_data.minutes);
                });
              });
              ui.heading("グループに含まれる画像");
              egui::ScrollArea::vertical().show(ui, |ui| {
                for photo_id in group_data.photo_id_list.iter() {
                  let photo_data = gui_photo_data_lst.get(photo_id).unwrap();
                  ui.horizontal(|ui| {
                    ui.label(format!("・{}（{}）", photo_data.photo_id, photo_data.alt));
                    let thumbnail = thumbnail_lst.get(photo_id).unwrap();
                    let thumbnail = image::compression(thumbnail, 65.0, 300).unwrap();
                    let image = RetainedImage::from_image_bytes(&*now_id, &thumbnail).unwrap();
                    image.show_size(ui, calculate_image_size(30.0, &image.size()));
                  });
                }
              });
              gui_group_data_lst.insert(now_id.clone(), group_data);
            }
          }
        }
      }
    });
  }
}

/// 適切な画像のサイズを計算する
fn calculate_image_size(max: f32, size: &[usize; 2]) -> egui::Vec2 {
  let width = size[0];
  let height = size[1];
  let x = if width > height {
    width as f32
  } else {
    height as f32
  };
  egui::vec2(width as f32 * (max / x), height as f32 * (max / x))
}

/// PhotoDataをJSON文字列に変換する
pub fn make_photo_data_json_str(
  photo_id_lst: &[String],
  photo_data_lst: &HashMap<String, GUIPhotoData>,
) -> String {
  let mut v = Vec::new();
  for photo_id in photo_id_lst.iter() {
    v.push(photodata::gui_photo_data_to_photo_data(
      photo_data_lst.get(photo_id).unwrap().clone(),
    ))
  }
  serde_json::to_string_pretty(&v).unwrap()
}

/// ImportPhotoDataをJSON文字列に変換する
pub fn make_import_photo_data_json_str(
  photo_id_lst: &[String],
  photo_data_lst: &HashMap<String, GUIPhotoData>,
) -> String {
  let mut v = Vec::new();
  for photo_id in photo_id_lst.iter() {
    v.push(photodata::gui_photo_data_to_import_photo_data(
      photo_data_lst.get(photo_id).unwrap().clone(),
    ))
  }
  serde_json::to_string_pretty(&v).unwrap()
}

/// GroupDataをJSON文字列に変換する
pub fn make_group_data_json_str(
  photo_id_lst: &[String],
  group_data_lst: &HashMap<String, GUIGroupData>,
) -> String {
  let mut v = Vec::new();
  for photo_id in photo_id_lst.iter() {
    v.push(photodata::gui_group_data_to_group_data(
      group_data_lst.get(photo_id).unwrap().clone(),
    ))
  }
  serde_json::to_string_pretty(&v).unwrap()
}

/// JSON文字列をファイルに書き出して保存する
pub fn save_json_str(json_str: String, path: &str) {
  let mut file = File::create(path).unwrap();
  let buf = json_str.into_bytes();
  file.write_all(&buf).unwrap();
  file.flush().unwrap();
}

/// 与えられた写真のIDがグループに含まれるかどうかを検索する
fn make_group_check_lst(
  now_id: &str,
  group_id_lst: &[String],
  gui_group_data_lst: &HashMap<String, GUIGroupData>,
) -> Vec<(String, bool)> {
  let mut v = Vec::new();
  for group_id in group_id_lst.iter() {
    let photo_id_lst = gui_group_data_lst
      .get(group_id)
      .map(|group_data| &group_data.photo_id_list)
      .unwrap();
    v.push((group_id.clone(), photo_id_lst.iter().any(|id| id == now_id)))
  }
  v
}

/// checkboxへの入力を元にグループデータを更新する
/// 新しく写真が追加されたグループを最新に持ってくるようにする
fn update_group_data(
  photo_id: &str,
  check_lst: &[(String, bool)],
  group_id_lst: &mut Vec<String>,
  gui_group_data_lst: &mut HashMap<String, GUIGroupData>,
) {
  let mut update_group_id_lst = Vec::new(); // 変更があったグループを溜める
  for (group_id, is_check) in check_lst.iter() {
    let group_data = gui_group_data_lst.get(group_id).unwrap();
    let mut photo_id_lst = group_data.clone().photo_id_list;
    if photo_id_lst.iter().any(|id| id == photo_id) {
      // IDが含まれている場合
      // is_checkがfalseのときにIDを削除する
      if !*is_check {
        photo_id_lst = photo_id_lst
          .iter()
          .filter(|id| id.to_string() != *photo_id)
          .cloned()
          .collect::<Vec<String>>();
      }
    } else {
      // IDが含まれていない場合
      // is_checkがtrueのときにIDを追加する
      // ついでに
      if *is_check {
        photo_id_lst.push(photo_id.to_owned());
        update_group_id_lst.push(group_id.clone());
      }
    }
    gui_group_data_lst.insert(
      group_id.clone(),
      GUIGroupData {
        photo_id_list: photo_id_lst.clone(),
        ..group_data.clone()
      },
    );
  }
  // 更新する
  for group_id in group_id_lst.iter() {
    if update_group_id_lst.iter().all(|id| id != group_id) {
      update_group_id_lst.push(group_id.to_string())
    }
  }
  *group_id_lst = update_group_id_lst;
}

/// 遅延読み込み用に使うかなり圧縮した画像を生成する
/// convertコマンドを動かすだけ
/// WindowsではWSLを経由してconvertコマンドを実行する
fn save_image_compression_lazy(original_raw_data: &[u8], output_path: &str) {
  let image_buf = image::compression(original_raw_data, 75.0, 32).unwrap();
  let mut file = File::create(output_path).unwrap();
  file.write_all(&image_buf).unwrap();
  file.flush().unwrap();
}

/// 実際に表示するためのやや圧縮した画像を生成する
/// convertコマンドを動かすだけ
/// WindowsではWSLを経由してconvertコマンドを実行する
fn save_image_compression_normal(original_raw_data: &[u8], output_path: &str) {
  let image_buf = image::compression(original_raw_data, 85.0, 2048).unwrap();
  let mut file = File::create(output_path).unwrap();
  file.write_all(&image_buf).unwrap();
  file.flush().unwrap();
}

/// ファイル系の保存
fn save_file(
  photo_id_lst: &[String],
  gui_photo_data_lst: &HashMap<String, GUIPhotoData>,
  group_id_lst: &[String],
  gui_group_data_lst: &HashMap<String, GUIGroupData>,
  input_json_path: &str,
  work_directory_path: &str,
) {
  // PhotoDataを保存
  let photo_data_json_str = make_photo_data_json_str(photo_id_lst, gui_photo_data_lst);
  let photo_data_json_path = format!("{}/photo_data.json", work_directory_path);
  save_json_str(photo_data_json_str, &photo_data_json_path);
  // GroupDataを保存
  let group_data_json_str = make_group_data_json_str(group_id_lst, gui_group_data_lst);
  let group_data_json_path = format!("{}/group_data.json", work_directory_path);
  save_json_str(group_data_json_str, &group_data_json_path);
  // ImportPhotoDataを保存
  let group_data_json_str = make_import_photo_data_json_str(photo_id_lst, gui_photo_data_lst);
  save_json_str(group_data_json_str, input_json_path);
}
