CREATE TABLE IF NOT EXISTS quotes(
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  author text NOT NULL,
  quote text NOT NULL,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  version int NOT NULL DEFAULT 1
);

