
use {
    crate::database::schema::*,
    std::borrow::Cow,
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

#[derive(Queryable, Insertable)]
#[table_name="feeds"]
pub struct Feed<'a> {
    pub name:        Cow<'a, str>,
    pub uri:         Cow<'a, str>,
    pub backlog:     i32,
    pub fetch_since: Option<NaiveDateTime>,
}

#[derive(Queryable, Insertable)]
#[table_name="register"]
pub struct Registration<'a> {
    pub feed: Cow<'a, str>,
    pub guid: Cow<'a, str>,
}

