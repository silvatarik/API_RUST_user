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

type RepositoryResultOutput<T> = Result<T,RepositoryError>;
//type RepositoryResult<'a ,T> = Pin<Box<dyn Future<Output =RepositoryResultOutput<T>> + 'a >>;
type RepositoryResult<'a ,T> = BoxFuture<'a , RepositoryResultOutput<T>>;

pub trait Repository: Send + Sync + 'static {
    fn get_users<'a>(&'a self, user_id: &'a Uuid) -> RepositoryResult<'a ,User>;
    fn create_user<'a>(&'a self, user: &'a User) -> RepositoryResult<'a ,User>;
    fn update_user<'a>(&'a self, user_id: &'a User) -> RepositoryResult<'a ,User>;
    fn delete_users<'a>(&'a self, user_id: &'a Uuid) -> RepositoryResult<'a ,Uuid>;
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

// impl FromRequest for RepositoryInjector {
//     type Error = actix_web::Error;
//     type Future = Ready<Result<Self,Self::Error>>;
//     type Config = ();
//
//     fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
//
//         //Acceder al app_data y que nos traiga en el req si tiene un instancia de tipo
//         //self que en este caso es tipo RepositoryInjector
//         if let Some(injector) = req.app_data::<Self>(){
//             let owned_injector = injector.to_owned();
//             ready(Ok(owned_injector))
//         }else{
//             ready(Err(ErrorBadRequest("No injector provided")))
//         }
//     }
// }

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

impl Repository for MemoryRepository{
    fn get_users<'a>(&'a self, user_id: &'a uuid::Uuid) -> RepositoryResult<'a , User> {
        //Dos formas de hacerlo
        //Primera ----------------------------------
        // let r = (||{
        //     let users = self.users.read()?;
        //         users.iter()
        //         .find(|u| &u.id == user_id)
        //         .cloned()
        //         // Este metodo no hizo falta debido a que la declaracion de arriba es exactamente la misma
        //         // .map(|u| u.clone())
        //         .ok_or_else(|| RepositoryError::InvalidId)
        // })();
        //Box::pin(std::future::ready(r))

        //Segunda ----------------------------------
        let f = async move{
            let users = self.users.read()?;
                users.iter()
                .find(|u| &u.id == user_id)
                .cloned()
                // Este metodo no hizo falta debido a que la declaracion de arriba es exactamente la misma
                // .map(|u| u.clone())
                .ok_or_else(|| RepositoryError::InvalidId)
        };
        //Box::pin(f)
        f.boxed()
    }

    fn create_user<'a>(&'a self, user: &'a User) ->RepositoryResult<'a ,User> {
        async move {
            //Bloquea el thread hasta que termine lo que tenga que hacer
            // let old_user =  futures::executor::block_on(self.get_users(&user.id));
            // if old_user.is_ok(){
            //     return Err(RepositoryError::AlreadyExists);
            // }
            if self.get_users(&user.id).await.is_ok(){
                return Err(RepositoryError::AlreadyExists);
            }
            let mut new_user = user.to_owned();
            new_user.create_at = Some(Utc::now());
            let mut users = self.users.write().unwrap();
            users.push(new_user.clone());
            Ok(new_user)
        }.boxed()
        //Box::pin(futures::future::ready(r))
    }

    fn update_user<'a>(&'a self, user: &'a User) -> RepositoryResult<'a ,User> {
        // if self.get_users(&user.id).is_err(){
        //     return Err(RepositoryError::DoesNotExist);
        // }
        // let mut update_user = user.to_owned();
        // update_user.update_at = Some(Utc::now());
        // let mut users = self.users.write().unwrap();
        // users.retain(|u| u.id != user.id);
        // users.push(update_user.clone());
        // Ok(update_user)
        //Box::pin(std::future::ready(Ok(user.to_owned())))

        async move {
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
        }.boxed()
    }

    fn delete_users<'a>(&'a self, user_id: &'a Uuid) -> RepositoryResult<'a ,Uuid> {
        async move{
            let mut users = self.users.write()?;
            users.retain(|u| u.id != *user_id);
            Ok(user_id.to_owned())
        }.boxed()
    }
}