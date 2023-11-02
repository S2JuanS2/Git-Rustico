#[cfg(test)]
mod tests {
    use git::commands::init::git_init;
    use git::commands::branch::get_current_branch;
    use std::fs;

    #[test]
    fn read_current_branch_test(){
        
        let directory = "./repo_test";
        git_init(directory).expect("Error al iniciar el repositorio");
        let branch = get_current_branch(directory).expect("Error al encontrar la branch");

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");

        assert_eq!(branch, "main");
    }

}