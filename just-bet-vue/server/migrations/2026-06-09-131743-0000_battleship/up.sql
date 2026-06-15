-- Your SQL goes here
CREATE TABLE battleship (
   id INT8 PRIMARY KEY,
   belongs_to INT8 REFERENCES users(id) NOT NULL,
   state VARCHAR(30) NOT NULL,
   opponent INT8 REFERENCES users(id),
   winner INT8 REFERENCES users(id),
   turn INT8 REFERENCES users(id) NOT NULL,
   stake MONEY NOT NULL,
   pool MONEY NOT NULL,
   created TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
