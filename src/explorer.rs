use crate::rpc::RpcClient;

pub struct Explorer {
    rpc: RpcClient,
    target: u64,
}

impl Explorer {
    pub fn new(url: Option<String>, target: u64) -> Explorer {
        let url = url.unwrap_or_else(|| "http://127.0.0.1:8114".to_string());
        Explorer {
            rpc: RpcClient::new(&url),
            target,
        }
    }


    // fn collect(&self) -> Vec<IssuedCell> {

    // }
}
