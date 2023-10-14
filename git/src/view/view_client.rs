use gtk::prelude::*;
use crate::errors::GitError;

pub fn start_view() -> Result<(), GitError>{

    if gtk::init().is_err() {
        return Err(GitError::GtkFailedInitiliaze);
    }
    let glade_src = include_str!("git.glade");

    let builder = gtk::Builder::from_string(glade_src);

    let window: gtk::Window = match builder.object("window1") {
        Some(window) => window,
        None => {
            return Err(GitError::ObjectBuildFailed);
        }
    };
    let button: gtk::Button = match builder.object("button1") {
        Some(button) => button,
        None => {
            return Err(GitError::ObjectBuildFailed);
        }
    };
    let entry: gtk::Entry = match builder.object("entry1") {
        Some(entry) => entry,
        None => {
            return Err(GitError::ObjectBuildFailed);
        }
    };
    button.connect_clicked(move |_| {
        println!("Comando recibido: {}", entry.text());
        
    });

    window.show_all();

    gtk::main();

    Ok(())
    
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_start_view(){
        let result = start_view();

        assert!(result.is_ok());
    }
}
*/

