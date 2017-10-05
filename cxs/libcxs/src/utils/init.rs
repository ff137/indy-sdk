use indy::api::ErrorCode as indyError;
use utils::error;


pub fn indy_error_to_cxs_error_code(err:indyError) ->  u32 {
    match err {
        indyError::Success => error::SUCCESS.code_num,
        _ => error::UNKNOWN_ERROR.code_num,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_error(){
        let indy_error = indyError::Success;
        let cxs_error = &error::SUCCESS;
        assert_eq!(indy_error_to_cxs_error_code(indy_error), cxs_error.code_num);

        let indy_error = indyError::WalletAlreadyExistsError;
        let cxs_error = &error::UNKNOWN_ERROR;
        assert_eq!(indy_error_to_cxs_error_code(indy_error), cxs_error.code_num);

    }


}