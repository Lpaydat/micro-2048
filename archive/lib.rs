use async_graphql::{Request, Response};
use linera_sdk::linera_base_types::{ContractAbi, ServiceAbi};

pub struct Game2048Abi;

impl ContractAbi for Game2048Abi {
    type Operation = u64;
    type Response = u64;
}

impl ServiceAbi for Game2048Abi {
    type Query = Request;
    type QueryResponse = Response;
}