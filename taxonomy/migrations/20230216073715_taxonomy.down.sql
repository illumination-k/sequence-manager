-- Add down migration script here
DROP INDEX taxonomy_name_index;

DROP TABLE taxonomy_record;
DROP TABLE taxonomy_relation;
