use std::fmt::Write as _;

use super::validated;

pub fn render(theme: &validated::Theme) -> String {
    let mut output = String::new();
    output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    output.push_str("<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n");
    output.push_str("<plist version=\"1.0\">\n");
    output.push_str("<dict>\n");
    write_key_string(&mut output, 1, "name", &theme.name);
    output.push_str("    <key>settings</key>\n");
    output.push_str("    <array>\n");
    output.push_str("        <dict>\n");
    output.push_str("            <key>settings</key>\n");
    output.push_str("            <dict>\n");
    write_key_string(&mut output, 4, "foreground", &theme.settings.foreground);
    write_key_string(&mut output, 4, "background", &theme.settings.background);
    write_key_string(
        &mut output,
        4,
        "lineHighlight",
        &theme.settings.line_highlight,
    );
    output.push_str("            </dict>\n");
    output.push_str("        </dict>\n");

    for rule in &theme.rules {
        output.push_str("        <dict>\n");
        if let Some(name) = &rule.name {
            write_key_string(&mut output, 3, "name", name);
        }
        write_key_string(&mut output, 3, "scope", &rule.scope);
        output.push_str("            <key>settings</key>\n");
        output.push_str("            <dict>\n");
        if let Some(foreground) = &rule.foreground {
            write_key_string(&mut output, 4, "foreground", foreground);
        }
        if let Some(background) = &rule.background {
            write_key_string(&mut output, 4, "background", background);
        }
        if let Some(font_style) = &rule.font_style {
            write_key_string(&mut output, 4, "fontStyle", &font_style.join(" "));
        }
        output.push_str("            </dict>\n");
        output.push_str("        </dict>\n");
    }

    output.push_str("    </array>\n");
    output.push_str("</dict>\n");
    output.push_str("</plist>\n");
    output
}

fn write_key_string(output: &mut String, indent: usize, key: &str, value: &str) {
    let padding = "    ".repeat(indent);
    let escaped = xml_escape(value);
    let _ = writeln!(output, "{padding}<key>{key}</key>");
    let _ = writeln!(output, "{padding}<string>{escaped}</string>");
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
