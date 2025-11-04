use crate::domain::*;
use crate::ports::RepositoryError;
use crate::usecases::MessageService;
use actix_web::{HttpResponse, Responder, delete, get, patch, post, web};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub svc: MessageService,
}

#[get("/messages/{channel}/{id}")]
async fn get_message(
    path: web::Path<(String, String)>,
    data: web::Data<AppState>,
) -> impl Responder {
    let (channel, id) = path.into_inner();
    let id = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => {
            return HttpResponse::BadRequest().json(ApiError {
                error: Some("invalid uuid".into()),
                code: Some("invalid_id".into()),
            });
        }
    };

    match data.svc.get_message(&channel, id).await {
        Ok(msg) => HttpResponse::Ok().json(msg),
        Err(RepositoryError::NotFound) => HttpResponse::NotFound().json(ApiError {
            error: Some("not found".into()),
            code: Some("not_found".into()),
        }),
        Err(RepositoryError::Forbidden) => HttpResponse::Forbidden().json(ApiError {
            error: Some("forbidden".into()),
            code: Some("forbidden".into()),
        }),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(Deserialize)]
struct ListQuery {
    limit: Option<u32>,
    before: Option<String>,
}

#[get("/messages/{channel}")]
async fn list_messages(
    path: web::Path<String>,
    q: web::Query<ListQuery>,
    data: web::Data<AppState>,
) -> impl Responder {
    let channel = path.into_inner();
    let before = match &q.before {
        Some(s) => match Uuid::parse_str(s) {
            Ok(u) => Some(u),
            Err(_) => {
                return HttpResponse::BadRequest().json(ApiError {
                    error: Some("invalid before uuid".into()),
                    code: Some("invalid_before".into()),
                });
            }
        },
        None => None,
    };

    match data.svc.list_messages(&channel, q.limit, before).await {
        Ok((items, next_before)) => {
            HttpResponse::Ok().json(serde_json::json!({"items": items, "next_before": next_before}))
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[patch("/messages/{id}")]
async fn patch_message(
    path: web::Path<String>,
    body: web::Json<MessageUpdate>,
    data: web::Data<AppState>,
) -> impl Responder {
    let id = match Uuid::parse_str(&path.into_inner()) {
        Ok(u) => u,
        Err(_) => {
            return HttpResponse::BadRequest().json(ApiError {
                error: Some("invalid uuid".into()),
                code: Some("invalid_id".into()),
            });
        }
    };

    match data.svc.update_message(id, body.into_inner()).await {
        Ok(msg) => HttpResponse::Ok().json(msg),
        Err(RepositoryError::NotFound) => HttpResponse::NotFound().json(ApiError {
            error: Some("not found".into()),
            code: Some("not_found".into()),
        }),
        Err(RepositoryError::Forbidden) => HttpResponse::Forbidden().json(ApiError {
            error: Some("forbidden".into()),
            code: Some("forbidden".into()),
        }),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/messages/{id}")]
async fn delete_message(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let id = match Uuid::parse_str(&path.into_inner()) {
        Ok(u) => u,
        Err(_) => {
            return HttpResponse::BadRequest().json(ApiError {
                error: Some("invalid uuid".into()),
                code: Some("invalid_id".into()),
            });
        }
    };

    match data.svc.delete_message(id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(RepositoryError::NotFound) => HttpResponse::NotFound().json(ApiError {
            error: Some("not found".into()),
            code: Some("not_found".into()),
        }),
        Err(RepositoryError::Forbidden) => HttpResponse::Forbidden().json(ApiError {
            error: Some("forbidden".into()),
            code: Some("forbidden".into()),
        }),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/messages/pin/{id}")]
async fn pin_message(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let id = match Uuid::parse_str(&path.into_inner()) {
        Ok(u) => u,
        Err(_) => {
            return HttpResponse::BadRequest().json(ApiError {
                error: Some("invalid uuid".into()),
                code: Some("invalid_id".into()),
            });
        }
    };

    match data.svc.pin_message(id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(RepositoryError::NotFound) => HttpResponse::NotFound().json(ApiError {
            error: Some("not found".into()),
            code: Some("not_found".into()),
        }),
        Err(RepositoryError::Forbidden) => HttpResponse::Forbidden().json(ApiError {
            error: Some("forbidden".into()),
            code: Some("forbidden".into()),
        }),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(Deserialize)]
struct PinsQuery {
    limit: Option<u32>,
    offset: Option<u32>,
}

#[get("/messages/pin/{channel}")]
async fn list_pins(
    path: web::Path<String>,
    q: web::Query<PinsQuery>,
    data: web::Data<AppState>,
) -> impl Responder {
    let channel = path.into_inner();
    match data.svc.list_pins(&channel, q.limit, q.offset).await {
        Ok((items, total)) => {
            HttpResponse::Ok().json(serde_json::json!({"items": items, "total": total}))
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(Deserialize)]
struct SearchQuery {
    channel: String,
    q: String,
    limit: Option<u32>,
    offset: Option<u32>,
    in_docs: Option<bool>,
}

#[get("/messages/search")]
async fn search(q: web::Query<SearchQuery>, data: web::Data<AppState>) -> impl Responder {
    match data
        .svc
        .search(&q.channel, &q.q, q.limit, q.offset, q.in_docs)
        .await
    {
        Ok((items, total)) => {
            HttpResponse::Ok().json(serde_json::json!({"items": items, "total": total}))
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig, svc: MessageService) {
    let state = AppState { svc };
    cfg.app_data(web::Data::new(state))
        .service(get_message)
        .service(list_messages)
        .service(patch_message)
        .service(delete_message)
        .service(pin_message)
        .service(list_pins)
        .service(search);
}
