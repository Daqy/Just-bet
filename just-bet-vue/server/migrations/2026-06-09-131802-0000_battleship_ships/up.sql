-- Your SQL goes here
CREATE TABLE battleship_ships (
  id INT8 PRIMARY KEY,
  belongs_to INT8 REFERENCES battleship(id) NOT NULL,
  placed_by INT8 REFERENCES users(id) NOT NULL,
  position INT8 NOT NULL,
  size INT8 NOT NULL
);
