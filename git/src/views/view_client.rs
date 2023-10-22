use crate::errors::GitError;
use crate::controllers::controller_client::Controller;
use gtk::prelude::*;

pub struct View{
    controller: Controller,
}

impl View{
    pub fn new(controller: Controller) -> View{
        View{
            controller,
        }
    }
    
    pub fn start_view(self) -> Result<(), GitError> {
        if gtk::init().is_err() {
            return Err(GitError::GtkFailedInitiliaze);
        }
        let glade_src = include_str!("git_ppal.glade");
    
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
            let command = entry.text().to_string();
            let _ = self.controller.send_command(command);
        });
    
        window.show_all();
    
        gtk::main();
    
        Ok(())
    }
    
}
