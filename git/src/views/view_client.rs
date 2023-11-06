use crate::controllers::controller_client::Controller;
use crate::errors::GitError;
use crate::views::buttons::*;
use crate::views::entries::*;
use gtk::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const DIV: &str = "\nResponse: ";

#[derive(Clone)]
pub struct View {
    controller: Rc<RefCell<Controller>>,
    window: gtk::Window,
    window_dialog_clone: gtk::Window,
    window_dialog_cat_file: gtk::Window,
    window_dialog_hash_object: gtk::Window,
    buttons: HashMap<String, gtk::Button>,
    entries: HashMap<String, Rc<gtk::Entry>>,
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

        let buttons_ids = get_buttons();
        let entries_ids = get_entries();

        let mut buttons: HashMap<String, gtk::Button> = HashMap::new();
        for button_id in buttons_ids {
            let button: gtk::Button = builder
                .object(&button_id)
                .ok_or(GitError::ObjectBuildFailed)?;
            buttons.insert(button_id, button);
        }
        let mut entries: HashMap<String, Rc<gtk::Entry>> = HashMap::new();

        for entry_id in entries_ids {
            let entry: Rc<gtk::Entry> = Rc::new(
                builder
                    .object(&entry_id)
                    .ok_or(GitError::ObjectBuildFailed)?,
            );
            entries.insert(entry_id, entry);
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
        let response: Rc<gtk::TextView> = Rc::new(
            builder
                .object("console")
                .ok_or(GitError::ObjectBuildFailed)?,
        );
        let label_user: gtk::Label = builder.object("user").ok_or(GitError::ObjectBuildFailed)?;

        let controller = Rc::new(RefCell::new(controller));
        Ok(View {
            controller,
            window,
            window_dialog_clone,
            window_dialog_hash_object,
            window_dialog_cat_file,
            buttons,
            entries,
            response,
            label_user,
        })
    }

    fn set_label_user(&mut self) {
        let controller = Rc::clone(&self.controller);
        let binding = controller.borrow_mut();
        let user_name = binding.get_name_client();
        self.label_user.set_text(user_name);
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
    fn connect_button_cmd(
        &mut self,
        entry_cmd: &str,
        button_cmd: &str,
        git_cmd: String,
        window: gtk::Window,
    ) {
        let controller = Rc::clone(&self.controller);
        let response = Rc::clone(&self.response);
        if let Some(entry) = self.entries.get(entry_cmd) {
            let entry_clone = Rc::clone(entry);
            if let Some(button) = self.buttons.get(button_cmd) {
                button.connect_clicked(move |_| {
                    window.hide();
                    let entry_format = format!("{} {}", git_cmd, entry_clone.text().to_string());
                    entry_clone.set_text("");
                    let result = controller.borrow_mut().send_command(&entry_format);
                    Self::response_write_buffer(result, Rc::clone(&response));
                });
            }
        };
    }
    fn connect_button_cat_file(&self) {
        let dialog = self.window_dialog_cat_file.clone();
        if let Some(button) = self.buttons.get(BUTTON_CAT_FILE) {
            button.connect_clicked(move |_| {
                dialog.show_all();
            });
        }
    }

    fn connect_button_hash_object(&self) {
        let dialog = self.window_dialog_hash_object.clone();

        if let Some(button) = self.buttons.get(BUTTON_HASH_OBJECT) {
            button.connect_clicked(move |_| {
                dialog.show_all();
            });
        }
    }
    fn connect_button_clone(&self) {
        let dialog = self.window_dialog_clone.clone();

        if let Some(button) = self.buttons.get(BUTTON_CLONE) {
            button.connect_clicked(move |_| {
                dialog.show_all();
            });
        }
    }
    fn connect_button_send(&self) {
        let response = Rc::clone(&self.response);
        let controller = Rc::clone(&self.controller);
        if let Some(entry) = self.entries.get(ENTRY_CONSOLE) {
            let entry_send = Rc::clone(entry);
            if let Some(button) = self.buttons.get(BUTTON_SEND) {
                button.connect_clicked(move |_| {
                    let command = entry_send.text().to_string();
                    entry_send.set_text("");
                    let result = controller.borrow_mut().send_command(&command);
                    Self::response_write_buffer(result, Rc::clone(&response));
                });
            }
        };
    }
    fn connect_button_clear(&self) {
        if let Some(button) = self.buttons.get(BUTTON_CLEAR) {
            if let Some(buffer) = self.response.buffer() {
                button.connect_clicked(move |_| {
                    buffer.set_text("");
                });
            }
        }
    }
    fn connect_button_with_entry(&self, entry_cmd: &str, button_cmd: &str, git_cmd: String) {
        let controller = Rc::clone(&self.controller);
        let response = Rc::clone(&self.response);
        if let Some(entry) = self.entries.get(entry_cmd) {
            let entry_branch = Rc::clone(entry);
            if let Some(button) = self.buttons.get(button_cmd) {
                button.connect_clicked(move |_| {
                    let entry_format = format!("{} {}", git_cmd, entry_branch.text().to_string());
                    entry_branch.set_text("");
                    let result = controller.borrow_mut().send_command(&entry_format);
                    Self::response_write_buffer(result, Rc::clone(&response));
                });
            }
        };
    }
    fn connect_button_with_not_entry(&self, button_cmd: &str, git_cmd: String) {
        let controller = Rc::clone(&self.controller);
        let response = Rc::clone(&self.response);

        if let Some(button) = self.buttons.get(button_cmd) {
            button.connect_clicked(move |_| {
                let result = controller.borrow_mut().send_command(&git_cmd);
                Self::response_write_buffer(result, Rc::clone(&response));
            });
        }
    }
    fn connect_buttons(&mut self) {
        self.connect_button_with_entry(ENTRY_CHECKOUT, BUTTON_CHECKOUT, "git checkout".to_string());
        self.connect_button_with_entry(ENTRY_ADD_RM, BUTTON_ADD, "git add".to_string());
        self.connect_button_with_entry(ENTRY_ADD_RM, BUTTON_RM, "git rm".to_string());
        self.connect_button_with_entry(ENTRY_COMMIT, BUTTON_COMMIT, "git commit".to_string());
        self.connect_button_with_entry(ENTRY_MERGE, BUTTON_MERGE, "git merge".to_string());
        self.connect_button_with_entry(ENTRY_BRANCH, BUTTON_BRANCH, "git branch".to_string());

        let commands = [
            (BUTTON_INIT, "git init".to_string()),
            (BUTTON_STATUS, "git status".to_string()),
            (BUTTON_PULL, "git pull".to_string()),
            (BUTTON_PUSH, "git push".to_string()),
            (BUTTON_FETCH, "git fetch".to_string()),
            (BUTTON_REMOTE, "git remote".to_string()),
            (BUTTON_LOG, "git log".to_string()),
        ];

        for (button, command) in commands {
            self.connect_button_with_not_entry(button, command);
        }

        self.connect_button_send();
        self.connect_button_clear();
        self.connect_button_clone();
        self.connect_button_cat_file();
        self.connect_button_hash_object();

        let window_clone = self.window_dialog_clone.clone();
        let window_cat_file = self.window_dialog_cat_file.clone();
        let window_hash_object = self.window_dialog_hash_object.clone();

        self.connect_button_cmd(
            ENTRY_CLONE,
            BUTTON_CMD_CLONE,
            "git clone".to_string(),
            window_clone,
        );
        self.connect_button_cmd(
            ENTRY_CAT_FILE,
            BUTTON_CMD_CAT_FILE,
            "git cat-file".to_string(),
            window_cat_file,
        );
        self.connect_button_cmd(
            ENTRY_HASH_OBJECT,
            BUTTON_CMD_HASH_OBJECT,
            "git hash-object".to_string(),
            window_hash_object,
        );
    }
    pub fn start_view(&mut self) -> Result<(), GitError> {
        self.connect_buttons();

        self.window.connect_destroy(|_| {
            gtk::main_quit();
        });

        self.set_label_user();

        self.window.show_all();
        gtk::main();

        Ok(())
    }
}
