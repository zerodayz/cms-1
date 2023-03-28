CREATE TABLE posts (
  id INT NOT NULL AUTO_INCREMENT,
  user_id INT NOT NULL,
  space_id INT NOT NULL,
  title VARCHAR(255) NOT NULL,
  title_description VARCHAR(255) NOT NULL,
  body TEXT NOT NULL,
  published BOOLEAN NOT NULL DEFAULT FALSE,
  updated_at DATETIME DEFAULT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (user_id) REFERENCES users(id),
  FOREIGN KEY (space_id) REFERENCES spaces(id)
);
