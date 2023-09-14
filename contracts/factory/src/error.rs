use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Identical Addresses")]
    IdenticalAddresses {}, 

    #[error("Empty Addresses")]
    EmptyAddresses {},

    #[error("Unable to fetch data in FACTORY_DATA")]
    FactoryDataFetchError {},

    #[error("Unable to update data in FACTORY_DATA")]
    FactoryDataUpdateError{},

    #[error("Reply id doesn't match")]
    ReplyIdError {},

    #[error("Unable to access reply data")]
    ReplyDataError {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

}
