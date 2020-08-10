CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE sessions (
       session_id uuid PRIMARY KEY DEFAULT uuid_generate_v4 (),
       app_id text NOT NULL,
       data jsonb NOT NULL
);
