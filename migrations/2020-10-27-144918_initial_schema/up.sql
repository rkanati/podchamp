pragma foreign_keys = on;

create table feeds(
    name        text     not null primary key,
    uri         text     not null,
    backlog     int      not null,
    fetch_since datetime
);

create table register(
    feed text not null references feeds(name) on delete cascade,
    guid text not null,
    primary key(feed, guid)
);

