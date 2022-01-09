-- Your SQL goes here
CREATE TABLE items
(
  id INTEGER AUTO_INCREMENT PRIMARY KEY NOT NULL,
  category_id INTEGER NOT NULL,
  FOREIGN KEY category_id_foreign_key (category_id) REFERENCES categories (id),
  hash VARCHAR(128) NOT NULL,
  name VARCHAR(1024) NOT NULL
);
