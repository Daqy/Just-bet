-- Your SQL goes here
CREATE TABLE clicks (
  id INT8 PRIMARY KEY,
  belongs_to INT8 REFERENCES minesweeper(id) NOT NULL,
  position INT8 NOT NULL,
  earned MONEY NOT NULL
);
