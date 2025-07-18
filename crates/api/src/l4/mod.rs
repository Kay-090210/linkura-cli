use crate::macros::{define_api_struct, use_common_crate};

pub mod account;
pub mod archive;
pub mod feslive;
pub mod user;
pub mod withlive;
pub mod petal_exchange;

use_common_crate!();
define_api_struct!(LinkuraApi);

impl<'a> LinkuraApi<'a> {
    pub fn account(&self) -> account::AccountApi {
        account::AccountApi { api: self }
    }

    pub fn user(&self) -> user::UserApi {
        user::UserApi { api: self }
    }

    pub fn archive(&self) -> archive::ArchiveApi {
        archive::ArchiveApi { api: self }
    }

    pub fn with_live(&self) -> withlive::WithLiveApi {
        withlive::WithLiveApi { api: self }
    }

    pub fn fes_live(&self) -> feslive::FesLiveApi {
        feslive::FesLiveApi { api: self }
    }

    pub fn petal_exchange(&self) -> petal_exchange::PetalExchangeApi {
        petal_exchange::PetalExchangeApi { api: self }
    }
}
