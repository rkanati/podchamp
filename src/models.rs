
use {
    crate::schema::*,
    chrono::prelude::*,
};

//struct StoredTimestamp(DateTime<Utc>);
//
//impl Into<DateTime<Utc>> for StoredTimestamp {
//    fn into(self) -> DateTime<Utc> { self.0 }
//}
//
//impl Queryable<S, B> for StoredTimestamp where
//    B: Backend,
//    String: Queryable<S, B>
//{
//    type Row = <String as Queryable<S, B>>::Row;
//
//    fn build(row: Self::Row) -> Self {
//
//    }
//}

#[derive(Queryable)]
pub struct Feed {
    pub name:        String,
    pub uri:         String,
    pub backlog:     i32,
    pub fetch_since: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[table_name="feeds"]
pub struct NewFeed<'a> {
    pub name:        &'a str,
    pub uri:         &'a str,
    pub backlog:     i32,
    pub fetch_since: Option<NaiveDateTime>,
}

#[derive(Queryable)]
pub struct Registration {
    pub feed: String,
    pub guid: String,
}

#[derive(Insertable)]
#[table_name="register"]
pub struct NewRegistration<'a> {
    pub feed: &'a str,
    pub guid: &'a str,
}

