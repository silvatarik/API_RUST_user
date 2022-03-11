use std::future::{Future, Ready, ready};
use std::ops::Deref;
use std::pin::Pin;
use std::sync::{Arc, PoisonError, RwLock};
use actix_web::{FromRequest, HttpRequest};
use actix_web::dev::Payload;
use actix_web::error::ErrorBadRequest;
use chrono::Utc;
use futures::future::BoxFuture;
use futures::FutureExt;
use uuid::Uuid;
use crate::user::User;
use async_trait::async_trait;

//Errores Personalizados
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("PoisonError: `{0}`")]
    LockError(String),
    #[error("This entity already exists")]
    AlreadyExists,
    #[error("This entity doesn t exists")]
    DoesNotExist,
    #[error("INVALID ID")]
    InvalidId
}

//Convertir poisonError a tipo RepositoryError
impl <T> From<PoisonError<T>> for RepositoryError {
    fn from(poisonError: PoisonError<T>) -> Self {
        RepositoryError::LockError(poisonError.to_string())
    }
}

type RepositoryResult<T> =  Result<T,RepositoryError>;

#[async_trait]
pub trait Repository: Send + Sync + 'static {
    async fn get_users(&self, user_id: &Uuid) -> RepositoryResult<User>;
    async fn create_user(&self, user: &User) -> RepositoryResult<User>;
    async fn update_user(&self, user_id: &User) -> RepositoryResult<User>;
    async fn delete_users(&self, user_id: &Uuid) -> RepositoryResult<Uuid>;
}

pub struct RepositoryInjector(Arc<Box<dyn Repository>>);

impl RepositoryInjector {
    pub fn new(repo: impl Repository) -> Self {
        Self(Arc::new(Box::new(repo)))
    }

    // pub fn new_shared(repo: impl Repository) -> Arc<Self> {
    //     Arc::new(Self::new(repo))
    // }
}

impl Clone for RepositoryInjector {
    fn clone(&self) -> Self {
        let repo = self.0.clone();
        Self(repo)
    }
}
impl Deref for RepositoryInjector {
    type Target = dyn Repository;

    fn deref(&self) -> &<Self as std::ops::Deref>::Target {

        self.0.as_ref().as_ref()
    }
}

pub struct MemoryRepository {
    users: RwLock<Vec<User>>,
}

impl Default for MemoryRepository {
    fn default() -> Self {
        Self {
            users:RwLock::new(vec![]),
            //User::new("Rob".to_string(),(1997,03,10))
        }
    }
}

#[async_trait]
impl Repository for MemoryRepository{
    async fn get_users(&self, user_id: &uuid::Uuid) -> RepositoryResult<User> {
        let users = self.users.read()?;
            users.iter()
            .find(|u| &u.id == user_id)
            .cloned()
            .ok_or_else(|| RepositoryError::InvalidId)

    }

    async fn create_user(&self, user: &User) ->RepositoryResult<User> {
        if self.get_users(&user.id).await.is_ok(){
            return Err(RepositoryError::AlreadyExists);
        }
        let mut new_user = user.to_owned();
        new_user.create_at = Some(Utc::now());
        let mut users = self.users.write().unwrap();
        users.push(new_user.clone());
        Ok(new_user)
    }

    async fn update_user(& self, user: &User) -> RepositoryResult<User> {
        if let Ok(old_user) = self.get_users(&user.id).await{
            let mut update_user = user.to_owned();
            update_user.update_at = old_user.create_at;
            let mut users = self.users.write().unwrap();
            users.retain(|u| u.id != user.id);
            users.push(update_user.clone());
            Ok(update_user)
        }else{
            Err(RepositoryError::DoesNotExist)
        }
    }

    async fn delete_users(&self, user_id: &Uuid) -> RepositoryResult<Uuid> {
        let mut users = self.users.write()?;
        users.retain(|u| u.id != *user_id);
        Ok(user_id.to_owned())
    }
}