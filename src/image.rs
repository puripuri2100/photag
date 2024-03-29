use anyhow::{anyhow, Result};
use image::{self, imageops::FilterType::Lanczos3, DynamicImage, RgbImage};
use mozjpeg::{ColorSpace, Compress, Decompress, Marker, ScanMode, ALL_MARKERS};
use std::fs;

pub fn open_file(path: &str) -> Result<Vec<u8>> {
  let raw_data = fs::read(path)?;
  Ok(raw_data)
}

pub fn compression(raw_data: &[u8], quality: f32, size: u32) -> Result<Vec<u8>> {
  let decomp = Decompress::with_markers(ALL_MARKERS).from_mem(raw_data)?;

  #[allow(clippy::needless_collect)]
  // markers の中に Exif 情報がある
  let markers: Vec<(Marker, Vec<u8>)> = decomp
    .markers()
    .into_iter()
    .map(|m| (m.marker, m.data.to_owned()))
    .collect();

  // RGB 形式でデコード開始
  let mut decomp_started = decomp.rgb()?;

  // 幅・高さ取得
  let width = decomp_started.width();
  let height = decomp_started.height();

  // デコードされたデータの取得
  let data = decomp_started
    .read_scanlines::<[u8; 3]>()
    .ok_or_else(|| anyhow!("read_scanlines error"))?
    .iter()
    .flatten()
    .cloned()
    .collect();

  // デコードの終了処理
  decomp_started.finish_decompress();

  // image crate の DynamicImage に変換
  let image_buffer = RgbImage::from_raw(width as u32, height as u32, data)
    .ok_or_else(|| anyhow!("from_raw error"))?;
  let img = DynamicImage::ImageRgb8(image_buffer);

  // リサイズとシャープ処理
  // 1) resize はアスペクトレシオを保持する
  // 2) unshrpen の一つ目の引数はどの程度ぼかしを入れるか（0.5~5.0 ぐらい？）
  // 　　二つ目の引数はしきい値（1~10 ぐらい？）
  // 　　どのぐらいの数値が良いかは画像によって変わる
  let img = img.resize(size, size, Lanczos3);

  // リサイズ後の幅・高さ取得
  let width = img.width() as usize;
  let height = img.height() as usize;

  // 変換後の RGB データ取得
  let data = img.into_rgb8().to_vec();

  // mozjpeg での圧縮処理
  let mut comp = Compress::new(ColorSpace::JCS_RGB);
  comp.set_scan_optimization_mode(ScanMode::AllComponentsTogether);
  comp.set_quality(quality);
  comp.set_size(width, height);
  comp.set_mem_dest();
  comp.start_compress();

  // Exif 情報を書き込む
  markers.into_iter().for_each(|m| {
    comp.write_marker(m.0, &m.1);
  });

  // RGB データを書き込む
  let mut line = 0;
  loop {
    if line > height - 1 {
      break;
    }
    let buf = unsafe { data.get_unchecked(line * width * 3..(line + 1) * width * 3) };
    comp.write_scanlines(buf);
    line += 1;
  }

  // 圧縮の終了処理
  comp.finish_compress();

  // ファイルに保存
  let buf = comp.data_to_vec().map_err(|e| anyhow!("{:?}", e))?;
  Ok(buf)
}
