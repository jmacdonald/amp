use preferences::AppInfo;

const APP_INFO: AppInfo = AppInfo{ name: "amp", author: "Jordan MacDonald" };

#[derive(RustcEncodable, RustcDecodable)]
pub struct Preferences {
    theme: String,
}

impl Preferences {
    pub fn new() -> Preferences {
        Preferences{ theme: String::from("solarized_dark") }
    }
}
