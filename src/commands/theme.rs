use commands::{self, Result};
use models::application::{Application, Mode};
use view;

pub fn use_selected_theme(app: &mut Application) -> Result {
    if let Mode::Theme(ref mut mode) = app.mode {
        let theme_key = mode.results.selection().ok_or("No theme selected")?;
        app.view.set_theme(&theme_key)?;

        // Persist the theme selection in the app preferences.
        app.preferences.insert(String::from(view::THEME_KEY), theme_key.clone());
        app.preferences.save()?;
    } else {
        bail!("Not in theme mode");
    };


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
