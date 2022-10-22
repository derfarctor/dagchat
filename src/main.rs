mod app;
mod crypto;
mod rpc;

use crate::app::components::title::ui::primary::*;
use crate::app::constants::VERSION;

fn main() {
    let backend_init = || -> std::io::Result<Box<dyn cursive::backend::Backend>> {
        let backend = cursive::backends::crossterm::Backend::init()?;
        let buffered_backend = cursive_buffered_backend::BufferedBackend::new(backend);
        Ok(Box::new(buffered_backend))
    };

    let mut siv = cursive::default();
    siv.set_window_title(format!("dagchat {}", VERSION));

    show_title(&mut siv);

    siv.try_run_with(backend_init).ok().unwrap();
}
