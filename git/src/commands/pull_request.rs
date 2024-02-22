use crate::api::{http_operation::HttpOperation, pr_info::PrInfo};

use super::errors::CommandsError;

pub fn handle_pr(_pr_info: PrInfo, _operation: HttpOperation) -> Result<String, CommandsError> {
    Ok("Soy un baby pr!".to_string())
}
