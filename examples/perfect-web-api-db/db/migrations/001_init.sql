create table accounts (
  id text primary key,
  email text not null unique,
  active boolean not null default true
);

