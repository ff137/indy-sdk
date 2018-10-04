use actix::prelude::*;
use actix_web::{AsyncResponder, FutureResponse, HttpResponse, State};
use actors::agency::{Agency, GetAgencyDetail, Post};
use futures::Future;

/// HttpExecutor state
pub struct AppState {
    pub agency: Addr<Agency>,
}

pub fn get(state: State<AppState>) -> FutureResponse<HttpResponse> {
    state.agency
        .send(GetAgencyDetail {})
        .from_err()
        .and_then(|res| match res {
            Ok(msg) => Ok(HttpResponse::Ok().json(msg)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

pub fn post_msg(state: State<AppState>) -> FutureResponse<HttpResponse> {
    state.agency
        .send(Post("Dummy message".to_owned()))
        .from_err()
        .and_then(|res| match res {
            Ok(msg) => Ok(HttpResponse::Ok().json(msg)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}