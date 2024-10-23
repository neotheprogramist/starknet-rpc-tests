use rand::prelude::SliceRandom;
use rand::thread_rng;
use url::Url;

use crate::v7::endpoints::errors::RpcError;
use crate::v7::endpoints::Rpc;

pub fn get_random_url(url_list: Vec<Url>) -> Result<Url, RpcError> {
    let mut rng = thread_rng();
    url_list
        .choose(&mut rng)
        .cloned()
        .ok_or_else(|| RpcError::EmptyUrlList("URL list is empty".to_string()))
}

pub fn set_random_rpc_url(rpc: &mut Rpc, url_list: Vec<Url>) -> Result<(), RpcError> {
    rpc.set_url(get_random_url(url_list)?);
    Ok(())
}
