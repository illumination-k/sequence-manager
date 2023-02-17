-- Add up migration script here
CREATE TABLE IF NOT EXISTS taxonomy_record (
    id text primary key not null,
    name text not null,
    rank text not null 
);

CREATE INDEX taxonomy_name_index on taxonomy_record(name);

CREATE TABLE IF NOT EXISTS taxonomy_relation (
    parent_id text not null references taxonomy_record (id),
    child_id text not null references taxonomy_record (id),
    rank text not null,
    primary key (parent_id, child_id)
);