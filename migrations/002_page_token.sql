CREATE TABLE IF NOT EXISTS page_tokens(
  token text PRIMARY KEY,
  next_page int NOT NULL
);

