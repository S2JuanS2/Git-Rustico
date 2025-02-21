#[cfg(test)]
mod tests {
    use git::commands::add::git_add;
    use git::commands::branch::get_current_branch;
    // use git::commands::commit::{git_commit, Commit};main
    use git::commands::init::git_init;
    // use git::commands::log::git_log;
    use git::commands::rm::git_rm;
    use git::commands::status::{get_index_content, git_status};
    use git::util::files::{open_file, read_file};
    use git::util::objects::builder_object_blob;
    use std::fs;
    use std::io::Write;

    #[test]
    fn read_current_branch_test() {
        let directory = "./repo_test";
        git_init(directory).expect("Error al iniciar el repositorio");
        let branch = get_current_branch(directory).expect("Error al encontrar la branch");

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");

        assert_eq!(branch, "master");
    }

    #[test]
    fn read_current_branch_test_fail() {
        let directory = "./branch_test";
        let branch = get_current_branch(directory);

        assert!(branch.is_err());
    }

    #[test]
    fn check_status_before_and_after_add_test() {
        let directory = "./testing_status";
        git_init(directory).expect("Error al iniciar el repositorio");
        let current_branch = get_current_branch(directory).expect("Error al encontrar la branch");

        let status_before_add = git_status(directory).expect("Error al obtener el status");
        let status_msg = format!("On branch {}\nYour branch is up to date with 'origin/{}'.\n\nnothing to commit, working tree clean\n", current_branch, current_branch);

        assert_eq!(status_before_add, status_msg);

        let file_path = format!("{}/{}", directory, "holamundo.txt");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Hola Mundo")
            .expect("Error al escribir en el archivo");

        git_add(directory, "holamundo.txt").expect("Error al agregar el archivo");

        let status_after_add = git_status(directory).expect("Error al obtener el status");
        let status_msg = format!("On branch {}\n\nChanges to be committed:\n  (use \"git reset HEAD <file>...\" to unstage)\n\n\tmodified:\t./testing_status/holamundo.txt\n", current_branch);

        assert_eq!(status_after_add, status_msg);

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");
    }

    #[test]
    fn adding_and_removing_files_test() {
        let directory = "./testing_add_remove";
        git_init(directory).expect("Error al iniciar el repositorio");

        let file_path = format!("{}/{}", directory, "toremove.txt");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Archivo a remover")
            .expect("Error al escribir en el archivo");

        git_add(directory, "toremove.txt").expect("Error al agregar el archivo");
        let file = open_file(&file_path).expect("Error al abrir el archivo");
        let content = read_file(file).expect("Error al leer el archivo");
        let git_dir = format!("{}/{}", directory, ".git");
        let hash_object =
            builder_object_blob(content, &git_dir).expect("Error al crear el objeto blob");
        let index_content_msg = format!("toremove.txt blob {}", hash_object);

        let index_content = get_index_content(&git_dir).expect("Error al leer el index");

        assert_eq!(index_content, index_content_msg);

        git_rm(directory, "toremove.txt").expect("Error al remover el archivo");

        let index_content = get_index_content(&git_dir).expect("Error al leer el index");

        assert_eq!(index_content, "");

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");
    }

    // #[test]
    // fn commit_and_log_test() {
    // let directory = "./testing_commit_log";
    // git_init(directory).expect("Error al iniciar el repositorio");
    // let git_dir = format!("{}/{}", directory, "/.git");

    // let file_path = format!("{}/{}", directory, "tocommit.txt");
    // let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
    // file.write_all(b"Archivo a commitear")
    // .expect("Error al escribir en el archivo");

    // let file_path2 = format!("{}/{}", directory, "othertocommit.txt");
    // let mut file2 = fs::File::create(&file_path2).expect("Falló al crear el archivo");
    // file2
    // .write_all(b"Otro archivo a commitear")
    // .expect("Error al escribir en el archivo");

    // git_add(directory, "tocommit.txt").expect("Error al agregar el archivo");

    // // let test_commit1 = Commit::new(
    // //     "prueba".to_string(),
    // //     "Valen".to_string(),
    // //     "vlanzillotta@fi.uba.ar".to_string(),
    // //     "Valen".to_string(),
    // //     "vlanzillotta@fi.uba.ar".to_string(),
    // // );
    // // git_commit(directory, test_commit1).expect("Error al commitear");
    // let current_branch = get_current_branch(directory).expect("Error al encontrar la branch");
    // let branch_current_path = format!("{}/{}/{}", git_dir, "refs/heads", current_branch);
    // let commit_hash1 =
    // fs::read_to_string(&branch_current_path).expect("Error al leer el archivo");
    // let author1 = "Valen".to_string();
    // let email1 = "vlanzillotta@fi.uba.ar".to_string();

    // git_add(directory, "othertocommit.txt").expect("Error al agregar el archivo");

    // // let test_commit2 = Commit::new(
    // //     "otra prueba".to_string(),
    // //     "Juan".to_string(),
    // //     "jdr@fi.uba.ar".to_string(),
    // //     "Juan".to_string(),
    // //     "jdr@fi.uba.ar".to_string(),
    // // );
    // // git_commit(directory, test_commit2).expect("Error al commitear");
    // let commit_hash2 =
    // fs::read_to_string(&branch_current_path).expect("Error al leer el archivo");
    // let author2 = "Juan".to_string();
    // let email2 = "jdr@fi.uba.ar".to_string();

    // let log = git_log(directory).expect("Error al obtener el log");
    // let log_msg = format!(
    // "Commit: {}\nAuthor: {} <{}>\nCommit: {}\nAuthor: {} <{}>\n",
    // commit_hash1, author1, email1, commit_hash2, author2, email2
    // );

    // println!("{}", log);
    // assert_eq!(log, log_msg);

    // fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");
    // }
}
