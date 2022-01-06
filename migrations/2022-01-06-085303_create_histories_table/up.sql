-- Your SQL goes here
CREATE TABLE histories
(
  id INTEGER AUTO_INCREMENT PRIMARY KEY NOT NULL,
  item_id INTEGER,
  foreign key item_id_foreign_key (item_id) REFERENCES items (id),
  price INTEGER,
  purchased_at VARCHAR(128)
);
