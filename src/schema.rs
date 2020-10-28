table! {
    feeds (name) {
        name -> Text,
        uri -> Text,
        backlog -> Integer,
        fetch_since -> Nullable<Timestamp>,
    }
}

table! {
    register (feed, guid) {
        feed -> Text,
        guid -> Text,
    }
}

joinable!(register -> feeds (feed));

allow_tables_to_appear_in_same_query!(
    feeds,
    register,
);
