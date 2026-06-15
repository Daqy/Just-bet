-- Your SQL goes here
CREATE TABLE minesweeper (
   id INT8 PRIMARY KEY,
   belongs_to INT8 REFERENCES users(id) NOT NULL,
   state VARCHAR(30) NOT NULL,
   result VARCHAR(30) NOT NULL,
   stake MONEY NOT NULL,
   pool MONEY NOT NULL,
   created TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
