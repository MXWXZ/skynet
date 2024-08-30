use std::fs;

use actix_web_validator::{Json, QsQuery};
use serde::{Deserialize, Serialize};
use skynet_api::{
    actix_cloud::{
        actix_web::{
            web::{Data, Path},
            Responder,
        },
        macros::partial_entity,
        response::{JsonResponse, RspResult},
        state::GlobalState,
    },
    entity::{groups, users::Column},
    finish,
    permission::{PermEntry, ROOT_ID},
    request::{
        unique_validator, Condition, IDsReq, IntoExpr, PageData, PaginationParam, Request,
        SortType, TimeParam,
    },
    sea_orm::{ColumnTrait, DatabaseConnection, IntoSimpleExpr, TransactionTrait},
    tracing::info,
    utils::{get_dataurl, parse_dataurl},
    HyUuid, Skynet,
};
use skynet_macro::common_req;
use validator::Validate;

use crate::{finish_data, finish_err, finish_ok, SkynetResponse};

#[common_req(Column)]
#[derive(Debug, Validate, Deserialize)]
pub struct GetReq {
    pub text: Option<String>,

    pub login_sort: Option<SortType>,
    #[validate(range(min = 0))]
    pub login_start: Option<i64>,
    #[validate(range(min = 0))]
    pub login_end: Option<i64>,

    #[serde(flatten)]
    #[validate(nested)]
    pub page: PaginationParam,
    #[serde(flatten)]
    #[validate(nested)]
    pub time: TimeParam,
}

pub async fn get_all(
    param: QsQuery<GetReq>,
    db: Data<DatabaseConnection>,
    skynet: Data<Skynet>,
) -> RspResult<impl Responder> {
    let mut cond = param.common_cond();
    if let Some(text) = &param.text {
        cond = cond.add(
            Condition::any()
                .add(text.like_expr(Column::Id))
                .add(text.like_expr(Column::Username))
                .add(text.like_expr(Column::LastIp)),
        );
    }
    cond = cond.add_option(param.login_start.map(|x| Column::LastLogin.gte(x)));
    cond = cond.add_option(param.login_start.map(|x| Column::LastLogin.lte(x)));
    if let Some(x) = param.login_sort {
        cond = cond.add_sort(Column::LastLogin.into_simple_expr(), x.into());
    };

    let (avatar, mime) = get_dataurl(&fs::read(&skynet.config.avatar)?);
    if mime.is_none() {
        finish_err!(SkynetResponse::UserInvalidAvatar);
    }
    let tx = db.begin().await?;
    let data = skynet.user.find(&tx, cond).await?;
    let data = (
        data.0
            .into_iter()
            .map(|mut x| {
                x.avatar = Some(
                    x.avatar
                        .map_or_else(|| avatar.clone().into(), |x| get_dataurl(&x).0.into()),
                );
                x
            })
            .collect(),
        data.1,
    );
    tx.commit().await?;
    finish_data!(PageData::new(data));
}

pub async fn get(
    uid: Path<HyUuid>,
    db: Data<DatabaseConnection>,
    skynet: Data<Skynet>,
) -> RspResult<impl Responder> {
    let tx = db.begin().await?;
    let data = skynet.user.find_by_id(&tx, &uid).await?;
    if data.is_none() {
        finish!(JsonResponse::not_found());
    }
    let mut data = data.unwrap();
    if data.avatar.is_none() {
        let (avatar, mime) = get_dataurl(&fs::read(&skynet.config.avatar)?);
        if mime.is_none() {
            finish_err!(SkynetResponse::UserInvalidAvatar);
        }
        data.avatar = Some(avatar.into());
    }
    tx.commit().await?;
    finish_data!(data);
}

#[derive(Debug, Validate, Deserialize)]
pub struct AddReq {
    #[validate(length(max = 32))]
    pub username: String,
    pub password: String,
    pub avatar: Option<String>,
    #[validate(custom(function = "unique_validator"))]
    pub group: Option<Vec<HyUuid>>,
    pub base: Option<HyUuid>,
    pub clone_group: Option<bool>,
}

pub async fn add(
    param: Json<AddReq>,
    db: Data<DatabaseConnection>,
    skynet: Data<Skynet>,
) -> RspResult<impl Responder> {
    if param.clone_group.is_some() && param.base.is_none() {
        finish!(JsonResponse::bad_request(
            "Base should not be None when clone group"
        ));
    }
    if param.base.is_some_and(|x| x.is_nil()) {
        finish_err!(SkynetResponse::UserRoot);
    }
    let avatar = if let Some(x) = &param.avatar {
        let (avatar, mime) = parse_dataurl(x);
        // 1MB
        if avatar.len() > 1024 * 1024 {
            finish!(JsonResponse::bad_request("File too large"));
        }
        if mime.is_none()
            || !["image/png", "image/jpeg", "image/webp"].contains(&mime.unwrap().mime_type())
        {
            finish_err!(SkynetResponse::UserInvalidAvatar);
        }
        Some(avatar)
    } else {
        None
    };
    let tx = db.begin().await?;
    if skynet
        .user
        .find_by_name(&tx, &param.username)
        .await?
        .is_some()
    {
        finish_err!(SkynetResponse::UserExist);
    }
    if let Some(group) = &param.group {
        for i in group {
            if skynet.group.find_by_id(&tx, i).await?.is_none() {
                finish_err!(SkynetResponse::GroupNotExist);
            }
        }
    }
    if let Some(x) = param.base {
        if skynet.user.find_by_id(&tx, &x).await?.is_none() {
            finish_err!(SkynetResponse::UserNotExist);
        }
    }

    let user = skynet
        .user
        .create(&tx, &param.username, Some(&param.password), avatar, false)
        .await?;
    if let Some(base) = &param.base {
        let perm: Vec<PermEntry> = skynet
            .perm
            .find_user(&tx, base)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
        skynet.perm.create_user(&tx, &user.id, &perm).await?;
    }
    let mut group: Vec<HyUuid> = param.group.as_ref().map_or(Vec::new(), ToOwned::to_owned);
    if param.clone_group.is_some_and(|x| x) {
        group.append(
            &mut skynet
                .group
                .find_user_group(&tx, &param.base.unwrap(), false)
                .await?
                .into_iter()
                .map(|x| x.id)
                .collect(),
        );
    }
    if !group.is_empty() {
        group.dedup();
        skynet.group.link(&tx, &[user.id], &group).await?;
    }
    tx.commit().await?;
    info!(
        success = true,
        username = param.username,
        uid = %user.id,
        gid = ?param.group,
        base = ?param.base,
        clone_group = param.clone_group,
        "Add user",
    );
    finish_data!(user.id);
}

#[derive(Debug, Validate, Deserialize)]
pub struct PutReq {
    #[validate(length(max = 32))]
    pub username: Option<String>,
    pub password: Option<String>,
    pub avatar: Option<String>,
    #[validate(custom(function = "unique_validator"))]
    pub group: Option<Vec<HyUuid>>,
}

pub async fn put(
    uid: Path<HyUuid>,
    param: Json<PutReq>,
    db: Data<DatabaseConnection>,
    state: Data<GlobalState>,
    req: Request,
    skynet: Data<Skynet>,
) -> RspResult<impl Responder> {
    if uid.is_nil()
        && (!req.uid.is_some_and(|x| x.is_nil())
            || param.group.as_ref().is_some_and(|x| !x.is_empty()))
    {
        finish_err!(SkynetResponse::UserRoot);
    }

    let tx = db.begin().await?;
    if let Some(user) = skynet.user.find_by_id(&tx, &uid).await? {
        if let Some(name) = &param.username {
            if let Some(x) = skynet.user.find_by_name(&tx, name).await? {
                if x.id != user.id {
                    finish_err!(SkynetResponse::UserExist);
                }
            }
        }
        if let Some(group) = &param.group {
            for i in group {
                if skynet.group.find_by_id(&tx, i).await?.is_none() {
                    finish_err!(SkynetResponse::GroupNotExist);
                }
            }
        }
        let avatar = if let Some(x) = &param.avatar {
            if x.is_empty() {
                Some(Vec::new())
            } else {
                let (avatar, mime) = parse_dataurl(x);
                // 1MB
                if avatar.len() > 1024 * 1024 {
                    finish!(JsonResponse::bad_request("File too large"));
                }
                if mime.is_none()
                    || !["image/png", "image/jpeg", "image/webp"]
                        .contains(&mime.unwrap().mime_type())
                {
                    finish_err!(SkynetResponse::UserInvalidAvatar);
                }
                Some(avatar)
            }
        } else {
            None
        };

        skynet
            .user
            .update(
                &tx,
                state.memorydb.clone(),
                &skynet,
                &user.id,
                param.username.as_deref(),
                param.password.as_deref(),
                avatar,
            )
            .await?;

        if let Some(gid) = &param.group {
            skynet.group.unlink(&tx, &[*uid], &[]).await?;
            skynet.group.link(&tx, &[*uid], gid).await?;
        }
    } else {
        finish!(JsonResponse::not_found());
    }
    tx.commit().await?;
    info!(
        success = true,
        username = param.username,
        uid = %uid,
        gid = ?param.group,
        "Put user",
    );
    finish_ok!();
}

pub async fn delete_batch(
    param: Json<IDsReq>,
    db: Data<DatabaseConnection>,
    state: Data<GlobalState>,
    req: Request,
    skynet: Data<Skynet>,
) -> RspResult<impl Responder> {
    if param.id.contains(&ROOT_ID) && !req.uid.is_some_and(|x| x.is_nil()) {
        finish_err!(SkynetResponse::UserRoot)
    }
    let tx = db.begin().await?;
    let rows = skynet
        .user
        .delete(&tx, state.memorydb.clone(), &skynet, &param.id)
        .await?;
    tx.commit().await?;
    if rows != 0 {
        info!(success = true, uid = ?param.id, "Delete users");
    }
    finish_data!(rows);
}

pub async fn delete(
    uid: Path<HyUuid>,
    db: Data<DatabaseConnection>,
    state: Data<GlobalState>,
    req: Request,
    skynet: Data<Skynet>,
) -> RspResult<impl Responder> {
    if uid.is_nil() && !req.uid.is_some_and(|x| x.is_nil()) {
        finish_err!(SkynetResponse::UserRoot);
    }
    let tx = db.begin().await?;
    if skynet.user.find_by_id(&tx, &uid).await?.is_none() {
        finish!(JsonResponse::not_found());
    }
    let rows = skynet
        .user
        .delete(&tx, state.memorydb.clone(), &skynet, &[*uid])
        .await?;
    tx.commit().await?;
    info!(success = true, uid = %uid, "Delete user");
    finish_data!(rows);
}

pub async fn kick(
    uid: Path<HyUuid>,
    db: Data<DatabaseConnection>,
    state: Data<GlobalState>,
    req: Request,
    skynet: Data<Skynet>,
) -> RspResult<impl Responder> {
    if uid.is_nil() && !req.uid.is_some_and(|x| x.is_nil()) {
        finish_err!(SkynetResponse::UserRoot);
    }
    let tx = db.begin().await?;
    if skynet.user.find_by_id(&tx, &uid).await?.is_none() {
        finish!(JsonResponse::not_found());
    }
    skynet
        .user
        .kick(state.memorydb.clone(), &skynet, &uid)
        .await?;
    tx.commit().await?;
    finish_ok!();
}

pub async fn get_group(
    uid: Path<HyUuid>,
    db: Data<DatabaseConnection>,
    skynet: Data<Skynet>,
) -> RspResult<impl Responder> {
    #[partial_entity(groups::Model)]
    #[derive(Serialize)]
    struct Rsp {
        pub id: HyUuid,
        pub name: String,
        pub created_at: i64,
        pub updated_at: i64,
    }

    let tx = db.begin().await?;
    if skynet.user.find_by_id(&tx, &uid).await?.is_none() {
        finish!(JsonResponse::not_found());
    }
    let data: Vec<Rsp> = skynet
        .group
        .find_user_group(&tx, &uid, true)
        .await
        .map(|x| (x.into_iter().map(Into::into).collect()))?;
    tx.commit().await?;
    finish_data!(data);
}
