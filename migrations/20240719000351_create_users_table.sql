-- Add migration script here

create table if not exists users
(
    id integer primary key not null,
    username text not null unique,
    password_hash text not null
);

-- Insert "ferris" user.
insert into users (id, username, password_hash)
values (1, 'jin', '$argon2id$v=19$m=19456,t=2,p=1$VE0e3g7DalWHgDwou3nuRA$uC6TER156UQpk0lNQ5+jHM0l5poVjPA1he/Tyn9J4Zw');
