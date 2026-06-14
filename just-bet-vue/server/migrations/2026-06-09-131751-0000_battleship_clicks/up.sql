-- Your SQL goes here
CREATE TABLE battleship_clicks (
  id INT8 PRIMARY KEY,
  belongs_to INT8 REFERENCES battleship(id) NOT NULL,
  clicked_by INT8 REFERENCES users(id) NOT NULL,
  boat_hit BOOLEAN NOT NULL DEFAULT FALSE,
  position INT8 NOT NULL
);
