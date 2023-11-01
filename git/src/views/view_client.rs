use crate::controllers::controller_client::Controller;
use crate::errors::GitError;
use gtk::prelude::*;
use std::rc::Rc;

const DIV: &str = "\nResponse: ";

#[derive(Clone)]
pub struct View {
    controller: Rc<Controller>,
    window: gtk::Window,
    window_dialog_clone: gtk::Window,
    window_dialog_cat_file: gtk::Window,
    window_dialog_hash_object: gtk::Window,
    buttons: Vec<gtk::Button>,
    entry_console: Rc<gtk::Entry>,
    entry_branch: Rc<gtk::Entry>,
    entry_add_rm: Rc<gtk::Entry>,
    entry_checkout: Rc<gtk::Entry>,
    entry_commit: Rc<gtk::Entry>,
    entry_merge: Rc<gtk::Entry>,
    entry_cmd_clone: Rc<gtk::Entry>,
    entry_cmd_hash_object: Rc<gtk::Entry>,
    entry_cmd_cat_file: Rc<gtk::Entry>,
    response: Rc<gtk::TextView>,
    label_user: gtk::Label,
}

impl View {
    pub fn new(controller: Controller) -> Result<View, GitError> {
        if gtk::init().is_err() {
            return Err(GitError::GtkFailedInitiliaze);
        }
        let glade_src = include_str!("git_ppal.glade");
        let builder = gtk::Builder::from_string(glade_src);

        let buttons_ids = vec!["button_clear", "button_send", "button_init", "button_branch", "button_status",
         "button_cat-file", "button_pull", "button_push", "button_fetch", "button_remote", "button_log", "button_hash-object", 
         "button_add", "button_rm", "button_checkout", "button_commit", "button_merge", "button_clone", "button_cmd_clone",
         "button_cmd_hash-object", "button_cmd_cat-file"];
        let mut buttons: Vec<gtk::Button> = vec![];

        for button_id in buttons_ids {
            let button: gtk::Button = builder.object(button_id).ok_or(GitError::ObjectBuildFailed)?;
            buttons.push(button);
        }
        
        let window: gtk::Window = builder
            .object("window_ppal")
            .ok_or(GitError::ObjectBuildFailed)?;
        let window_dialog_clone: gtk::Window = builder
            .object("window_dialog_clone")
            .ok_or(GitError::ObjectBuildFailed)?;
        let window_dialog_hash_object: gtk::Window = builder
            .object("window_dialog_hash-object")
            .ok_or(GitError::ObjectBuildFailed)?;
        let window_dialog_cat_file: gtk::Window = builder
            .object("window_dialog_cat-file")
            .ok_or(GitError::ObjectBuildFailed)?;
        let entry_console: Rc<gtk::Entry> = Rc::new(
            builder
                .object("entry_console")
                .ok_or(GitError::ObjectBuildFailed)?,
        );
        let entry_branch: Rc<gtk::Entry> = Rc::new(
            builder
                .object("entry_branch")
                .ok_or(GitError::ObjectBuildFailed)?,
        );
        let entry_checkout: Rc<gtk::Entry> = Rc::new(
            builder
                .object("entry_checkout")
                .ok_or(GitError::ObjectBuildFailed)?,
        );
        let entry_add_rm: Rc<gtk::Entry> = Rc::new(
            builder
                .object("entry_add_rm")
                .ok_or(GitError::ObjectBuildFailed)?,
        );
        let entry_commit: Rc<gtk::Entry> = Rc::new(
            builder
                .object("entry_commit")
                .ok_or(GitError::ObjectBuildFailed)?,
        );
        let entry_merge: Rc<gtk::Entry> = Rc::new(
            builder
                .object("entry_merge")
                .ok_or(GitError::ObjectBuildFailed)?,
        );
        let entry_cmd_clone: Rc<gtk::Entry> = Rc::new(
            builder
                .object("entry_clone")
                .ok_or(GitError::ObjectBuildFailed)?,
        );
        let entry_cmd_hash_object: Rc<gtk::Entry> = Rc::new(
            builder
                .object("entry_hash-object")
                .ok_or(GitError::ObjectBuildFailed)?,
        );
        let entry_cmd_cat_file: Rc<gtk::Entry> = Rc::new(
            builder
                .object("entry_cat-file")
                .ok_or(GitError::ObjectBuildFailed)?,
        );
        let response: Rc<gtk::TextView> = Rc::new(
            builder
                .object("console")
                .ok_or(GitError::ObjectBuildFailed)?,
        );
        let label_user: gtk::Label = builder
                .object("user")
                .ok_or(GitError::ObjectBuildFailed)?;

        let controller = Rc::new(controller);
        Ok(View {
            controller,
            window,
            window_dialog_clone,
            window_dialog_hash_object,
            window_dialog_cat_file,
            buttons,
            entry_console,
            entry_branch,
            entry_checkout,
            entry_add_rm,
            entry_commit,
            entry_merge,
            entry_cmd_clone,
            entry_cmd_hash_object,
            entry_cmd_cat_file,
            response,
            label_user,
        })

    }
    
    fn set_label_user(&mut self, name: &str){
        self.label_user.set_text(name);
    }

    fn response_write_buffer(result: Result<String, GitError>, response: Rc<gtk::TextView>) {
        if let Some(buffer) = response.buffer() {
            let mut end_iter = buffer.end_iter();
            match result {
                Ok(response) => {
                    let response_format = format!("{}\n{}", DIV, response);
                    buffer.insert(&mut end_iter, &response_format);
                }
                Err(e) => {
                    let error_message = format!(
                        "{}\nError al enviar el comando.\n[Error] {}\n",
                        DIV,
                        e.message()
                    );
                    buffer.insert(&mut end_iter, &error_message);
                }
            }
        }
    }
    fn connect_button_init(&self) {
        let controller = Rc::clone(&self.controller);
        let response = Rc::clone(&self.response);

        if let Some(button) = self.buttons.get(2){
            button.connect_clicked(move |_| {
                let result = controller.send_command("git init");
                Self::response_write_buffer(result, Rc::clone(&response));
            });
        }
    }

    fn connect_button_cmd_clone(&self){
        let dialog = self.window_dialog_clone.clone();
        let controller = Rc::clone(&self.controller);
        let entry = Rc::clone(&self.entry_cmd_clone);
        let response = Rc::clone(&self.response);
        
        if let Some(button) = self.buttons.get(18) {
            button.connect_clicked(move |_| {
                dialog.hide();
                let entry_format = format!("git clone {}", entry.text().to_string());
                entry.set_text("");
                let result = controller.send_command(&entry_format);
                Self::response_write_buffer(result, Rc::clone(&response));
            });
        }
    }
    fn connect_button_cmd_hash_object(&self){
        let dialog = self.window_dialog_hash_object.clone();
        let controller = Rc::clone(&self.controller);
        let entry = Rc::clone(&self.entry_cmd_hash_object);
        let response = Rc::clone(&self.response);
        
        if let Some(button) = self.buttons.get(19) {
            button.connect_clicked(move |_| {
                dialog.hide();
                let entry_format = format!("git hash-object {}", entry.text().to_string());
                entry.set_text("");
                let result = controller.send_command(&entry_format);
                Self::response_write_buffer(result, Rc::clone(&response));
            });
        }
    }
    fn connect_button_cmd_cat_file(&self){
        let dialog = self.window_dialog_cat_file.clone();
        let controller = Rc::clone(&self.controller);
        let entry = Rc::clone(&self.entry_cmd_cat_file);
        let response = Rc::clone(&self.response);
        
        if let Some(button) = self.buttons.get(20) {
            button.connect_clicked(move |_| {
                dialog.hide();
                let entry_format = format!("git cat-file {}", entry.text().to_string());
                entry.set_text("");
                let result = controller.send_command(&entry_format);
                Self::response_write_buffer(result, Rc::clone(&response));
            });
        }
    }

    fn connect_button_cat_file(& self) {
        let dialog = self.window_dialog_cat_file.clone();
        if let Some(button) = self.buttons.get(5) {
            button.connect_clicked(move |_| {
                dialog.show_all();
            });
        }
    }
    
    fn connect_button_hash_object(&self) {
        let dialog = self.window_dialog_hash_object.clone();

        if let Some(button) = self.buttons.get(11){
            button.connect_clicked(move |_| {
                dialog.show_all();
            });
        }
    }
    fn connect_button_clone(&self) {
        let dialog = self.window_dialog_clone.clone();

        if let Some(button) = self.buttons.get(17){
            button.connect_clicked(move |_| {
                dialog.show_all();
    
            });
        }
    }
    fn connect_button_clear(&self) {
        if let Some(button) = self.buttons.get(0){
            if let Some(buffer) = self.response.buffer() {
                button.connect_clicked(move |_| {
                    buffer.set_text("");
                });
            }
        }
    }
    fn connect_button_branch(&self) {
        let controller = Rc::clone(&self.controller);
        let entry = Rc::clone(&self.entry_branch);
        let response = Rc::clone(&self.response);

        if let Some(button) = self.buttons.get(3){
            button.connect_clicked(move |_| {
                let entry_format = format!("git branch {}", entry.text().to_string());
                entry.set_text("");
                let result = controller.send_command(&entry_format);
                Self::response_write_buffer(result, Rc::clone(&response));
            });
        }
    }
    fn connect_button_checkout(&self) {
        let controller = Rc::clone(&self.controller);
        let entry = Rc::clone(&self.entry_checkout);
        let response = Rc::clone(&self.response);

        if let Some(button) = self.buttons.get(14){
            button.connect_clicked(move |_| {
                let entry_format = format!("git checkout {}", entry.text().to_string());
                entry.set_text("");
                let result = controller.send_command(&entry_format);
                Self::response_write_buffer(result, Rc::clone(&response));
            });
        }
    }
    fn connect_button_add(&self) {
        let controller = Rc::clone(&self.controller);
        let entry = Rc::clone(&self.entry_add_rm);
        let response = Rc::clone(&self.response);

        if let Some(button) = self.buttons.get(12){
            button.connect_clicked(move |_| {
                let entry_format = format!("git add {}", entry.text().to_string());
                entry.set_text("");
                let result = controller.send_command(&entry_format);
                Self::response_write_buffer(result, Rc::clone(&response));
            });
        }
    }
    fn connect_button_rm(&self) {
        let controller = Rc::clone(&self.controller);
        let entry = Rc::clone(&self.entry_add_rm);
        let response = Rc::clone(&self.response);

        if let Some(button) = self.buttons.get(13){
            button.connect_clicked(move |_| {
                let entry_format = format!("git rm {}", entry.text().to_string());
                entry.set_text("");
                let result = controller.send_command(&entry_format);
                Self::response_write_buffer(result, Rc::clone(&response));
            });
        }
    }
    fn connect_button_commit(&self) {
        let controller = Rc::clone(&self.controller);
        let entry = Rc::clone(&self.entry_commit);
        let response = Rc::clone(&self.response);

        if let Some(button) = self.buttons.get(15){
            button.connect_clicked(move |_| {
                let entry_format = format!("git commit -m {}", entry.text().to_string());
                entry.set_text("");
                let result = controller.send_command(&entry_format);
                Self::response_write_buffer(result, Rc::clone(&response));
            });
        }
    }
    fn connect_button_merge(&self) {
        let controller = Rc::clone(&self.controller);
        let entry = Rc::clone(&self.entry_merge);
        let response = Rc::clone(&self.response);

        if let Some(button) = self.buttons.get(16){
            button.connect_clicked(move |_| {
                let entry_format = format!("git merge {}", entry.text().to_string());
                entry.set_text("");
                let result = controller.send_command(&entry_format);
                Self::response_write_buffer(result, Rc::clone(&response));
            });
        }
    }
    fn connect_button_send(&self) {
        let response = Rc::clone(&self.response);
        let entry = Rc::clone(&self.entry_console);
        let controller = Rc::clone(&self.controller);

        if let Some(button) = self.buttons.get(1){
            button.connect_clicked(move |_| {
                let command = entry.text().to_string();
                entry.set_text("");
                let result = controller.send_command(&command);
                Self::response_write_buffer(result, Rc::clone(&response));
            });
        }
    }
    fn connect_button_status(&self) {
        let response = Rc::clone(&self.response);
        let controller = Rc::clone(&self.controller);

        if let Some(button) = self.buttons.get(4){
            button.connect_clicked(move |_| {
                let result = controller.send_command("git status");
                Self::response_write_buffer(result, Rc::clone(&response));
            });
        }
    }
    fn connect_button_pull(&self) {
        let response = Rc::clone(&self.response);
        let controller = Rc::clone(&self.controller);

        if let Some(button) = self.buttons.get(6){
            button.connect_clicked(move |_| {
                let result = controller.send_command("git pull");
                Self::response_write_buffer(result, Rc::clone(&response));
            });
        }
    }
    fn connect_button_fetch(&self) {
        let response = Rc::clone(&self.response);
        let controller = Rc::clone(&self.controller);

        if let Some(button) = self.buttons.get(8){
            button.connect_clicked(move |_| {
                let result = controller.send_command("git fetch");
                Self::response_write_buffer(result, Rc::clone(&response));
            });
        }
    }
    fn connect_button_remote(&self) {
        let response = Rc::clone(&self.response);
        let controller = Rc::clone(&self.controller);

        if let Some(button) = self.buttons.get(9){
            button.connect_clicked(move |_| {
                let result = controller.send_command("git remote");
                Self::response_write_buffer(result, Rc::clone(&response));
            });
        }
    }
    fn connect_button_log(&self) {
        let response = Rc::clone(&self.response);
        let controller = Rc::clone(&self.controller);

        if let Some(button) = self.buttons.get(10){
            button.connect_clicked(move |_| {
                let result = controller.send_command("git log");
                Self::response_write_buffer(result, Rc::clone(&response));
            });
        }
    }
    fn connect_button_push(&self) {
        let response = Rc::clone(&self.response);
        let controller = Rc::clone(&self.controller);

        if let Some(button) = self.buttons.get(7){
            button.connect_clicked(move |_| {
                let result = controller.send_command("git push");
                Self::response_write_buffer(result, Rc::clone(&response));
            });
        }
    }
    fn connect_buttons(self) {
        self.connect_button_init();
        self.connect_button_clear();
        self.connect_button_send();
        self.connect_button_branch();
        self.connect_button_cat_file();
        self.connect_button_status();
        self.connect_button_pull();
        self.connect_button_push();
        self.connect_button_fetch();
        self.connect_button_remote();
        self.connect_button_log();
        self.connect_button_clone();
        self.connect_button_hash_object();
        self.connect_button_checkout();
        self.connect_button_add();
        self.connect_button_rm();
        self.connect_button_commit();
        self.connect_button_merge();
        self.connect_button_cmd_clone();
        self.connect_button_cmd_hash_object();
        self.connect_button_cmd_cat_file();

    }
    pub fn start_view(&mut self) -> Result<(), GitError> {
        let this = self.clone();
        this.connect_buttons();

        self.window.connect_destroy(|_| {
            gtk::main_quit();
        });

        let controller = Rc::clone(&self.controller);
        let user_name = controller.get_name_client();
        self.set_label_user(user_name);

        self.window.show_all();
        gtk::main();

        Ok(())
    }
}
