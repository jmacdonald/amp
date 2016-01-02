use models::application::Application;

pub fn add(app: &mut Application) {
    if let Some(ref mut repo) = app.repository {
        if let Some(buf) = app.workspace.current_buffer() {
            if let Ok(ref mut index) = repo.index() {
                if let Some(ref path) = buf.path {
                    index.add_path(&path);
                    index.write();
                }
            }
        }
    }
}
