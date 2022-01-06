-- Your SQL goes here
CREATE TABLE histories
(
  id INTEGER AUTO_INCREMENT PRIMARY KEY NOT NULL,
  item_id INTEGER NOT NULL,
  FOREIGN KEY item_id_foreign_key (item_id) REFERENCES items (id),
  price INTEGER NOT NULL,
  purchased_at VARCHAR(128) NOT NULL
);
