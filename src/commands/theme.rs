use commands::{self, Result};
use models::application::{Application, Mode};

pub fn use_selected_theme(app: &mut Application) -> Result {
    if let Mode::Theme(ref mut mode) = app.mode {
        let theme_key = mode
            .results
            .selection()
            .ok_or("No theme selected")?;
        app.view.set_theme(theme_key)?;
    }

    commands::view::scroll_cursor_to_center(app)?;
    commands::application::switch_to_normal_mode(app)
}

pub fn search(app: &mut Application) -> Result {
    if let Mode::Theme(ref mut mode) = app.mode {
        mode.search();
    } else {
        bail!("Can't search symbols outside of symbol jump mode");
    }

    Ok(())
}

pub fn select_next_symbol(app: &mut Application) -> Result {
    if let Mode::Theme(ref mut mode) = app.mode {
        mode.results.select_next();
    } else {
        bail!("Can't change symbol selection outside of symbol jump mode");
    }

    Ok(())
}

pub fn select_previous_symbol(app: &mut Application) -> Result {
    if let Mode::Theme(ref mut mode) = app.mode {
        mode.results.select_previous();
    } else {
        bail!("Can't change symbol selection outside of symbol jump mode");
    }

    Ok(())
}

pub fn enable_insert(app: &mut Application) -> Result {
    if let Mode::Theme(ref mut mode) = app.mode {
        mode.insert = true;
    } else {
        bail!("Can't change symbol search insert state outside of symbol jump mode");
    }

    Ok(())
}

pub fn disable_insert(app: &mut Application) -> Result {
    if let Mode::Theme(ref mut mode) = app.mode {
        mode.insert = false;
    } else {
        bail!("Can't change symbol search insert state outside of symbol jump mode");
    }

    Ok(())
}
