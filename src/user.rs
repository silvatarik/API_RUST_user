use chrono::{DateTime, NaiveDate, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct User{
    pub id:uuid::Uuid,
    pub name:String,
    pub birth_date:NaiveDate,
    pub custom_data: CustomData,
    pub create_at:Option<DateTime<Utc>>,
    pub update_at:Option<DateTime<Utc>>,
}

impl User {
    pub fn new(name:String, birth_date_ymd:(i32,u32,u32)) -> Self{
        let (year,month,day) = birth_date_ymd;
        let id = uuid::Uuid::parse_str("a50f1258-4ed3-4123-908d-679f48a1dd20")
            .unwrap();
        Self{
            id,
            name,
            birth_date: NaiveDate::from_ymd(year,month,day),
            custom_data: CustomData{random:1},
            create_at: Some(Utc::now()),
            update_at: None,
        }
    }
}

#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct CustomData{
    pub random:u32
}