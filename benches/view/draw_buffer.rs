extern crate amp;
#[macro_use]
extern crate criterion;

use amp::Application;
use criterion::Criterion;
use std::path::PathBuf;

fn buffer_rendering(c: &mut Criterion) {
    let mut app = Application::new().unwrap();
    app.workspace.open_buffer(
        &PathBuf::from("src/commands/buffer.rs")
    ).unwrap();
    app.view.initialize_buffer(app.workspace.current_buffer().unwrap()).unwrap();

    c.bench_function("buffer rendering", move |b| b.iter(|| {
        app.view.draw_buffer(
            app.workspace.current_buffer().unwrap(),
            None,
            None
        ).unwrap()
    }));
}

fn scrolled_buffer_rendering(c: &mut Criterion) {
    let mut app = Application::new().unwrap();
    app.workspace.open_buffer(
        &PathBuf::from("src/commands/buffer.rs")
    ).unwrap();
    app.view.initialize_buffer(app.workspace.current_buffer().unwrap()).unwrap();

    // Scroll to the bottom of the buffer.
    app.workspace.current_buffer().unwrap().cursor.move_to_last_line();
    app.view.scroll_to_cursor(app.workspace.current_buffer().unwrap()).unwrap();

    c.bench_function("scrolled buffer rendering", move |b| b.iter(|| {
        app.view.draw_buffer(
            app.workspace.current_buffer().unwrap(),
            None,
            None
        ).unwrap()
    }));
}

criterion_group!(benches, buffer_rendering, scrolled_buffer_rendering);
criterion_main!(benches);
