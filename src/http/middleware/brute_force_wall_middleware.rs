use std::net::{SocketAddr, ToSocketAddrs};

use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::{Logger, Next},
    Error,
};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;

use crate::{database::db::get_connection, http::GenericError, model::user::user::User, schema::users, services::auth::decode_jwt};

pub async fn brute_force_wall_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let requisicao = req.peer_addr().unwrap().ip();
    println!("{:#?}", requisicao);
    next.call(req).await
}