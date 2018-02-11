extern crate amp;
#[macro_use]
extern crate criterion;

use amp::Application;
use criterion::Criterion;
use std::path::PathBuf;

fn criterion_benchmark(c: &mut Criterion) {
    let mut app = Application::new().unwrap();
    app.workspace.open_buffer(
        &PathBuf::from("benches/view/draw_buffer.rs")
    );
    c.bench_function("draw_buffer", move |b| b.iter(|| {
        app.view.draw_buffer(
            app.workspace.current_buffer().unwrap(),
            None,
            None
        )
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
