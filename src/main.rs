//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use clap::Parser;

mod gui;
mod image;
mod photodata;
mod save;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
  /// 画像ファイル名が書かれたJSONファイルへのpath
  #[clap(short, long)]
  input: String,
  /// オリジナルの画像が置かれているフォルダへのpath
  #[clap(short, long)]
  original: String,
  /// 圧縮した画像ファイルやデータのJSONファイルを出力する作業ディレクトリへのpath
  #[clap(short, long)]
  work: String,
}

fn main() {
  let args = Args::parse();
  let native_options = eframe::NativeOptions {
    min_window_size: Some(eframe::egui::vec2(900.0, 900.0)),
    resizable: true,
    default_theme: eframe::Theme::Light,
    ..Default::default()
  };
  eframe::run_native(
    "photag",
    native_options,
    Box::new(|cc| {
      Box::new(gui::PhotagApp::new(
        cc,
        args.input,
        args.original,
        args.work,
      ))
    }),
  );
}
