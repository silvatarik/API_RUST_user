use uuid::Uuid;
use crate::user::User;

pub trait Repository {
    fn get_users(&self, user_id: uuid::Uuid) -> Result<User,String>;
}

pub struct MemoryRepository {
    users: Vec<User>,
}

impl Default for MemoryRepository {
    fn default() -> Self {
        Self {
            users:vec![User::new("Rob".to_string(),(1997,03,10))],
        }
    }
}

impl Repository for MemoryRepository{
    fn get_users(&self, user_id: Uuid) -> Result<User, String> {
        self.users.iter()
            .find(|u| u.id == user_id)
            //.cloned() Este metodo no hizo falta debido a que la declaracion de arriba es exactamente la misma
            .map(|u| u.clone())
            .ok_or_else(|| "Invalid UUID".to_string())//Para este clone se implemento el decorador de clone en el struc User
    }
}