mod compressor;
mod converter;
mod downloader;
mod extractor;
mod ffmpeg;
mod menu;
mod probe;
mod reformatter;
mod remux;
mod subtitles;
mod thumbnail;
#[tokio::main]
async fn main() -> std::io::Result<()> {
    for (_name, _path) in ffmpeg::ffmpeg_selection() {
        //println!("{name}: {}", path.display());
    }

    let mut terminal = ratatui::init();
    let result = menu::App::new().run(&mut terminal);
    ratatui::restore();
    result
}