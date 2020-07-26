CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE sessions (
       session_id uuid PRIMARY KEY DEFAULT uuid_generate_v4 (),
       app_id uuid NOT NULL,
       data jsonb NOT NULL
);

CREATE TABLE apps (
       app_id uuid PRIMARY KEY DEFAULT uuid_generate_v4 (),
       name text NOT NULL,
       token uuid NOT NULL DEFAULT uuid_generate_v4 ()
);

CREATE TABLE admin_tokens (
       token uuid PRIMARY KEY DEFAULT uuid_generate_v4 ()
);
