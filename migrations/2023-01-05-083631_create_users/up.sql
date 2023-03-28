CREATE TABLE users (
        id INT NOT NULL AUTO_INCREMENT,
        username VARCHAR(255) NOT NULL UNIQUE,
        password VARCHAR(255) NOT NULL,
        is_admin BOOLEAN NOT NULL,
        PRIMARY KEY (id)
)
