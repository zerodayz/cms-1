CREATE TABLE sessions (
    id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    user_id INT NOT NULL,
    session_id VARCHAR(255) NOT NULL,
    start_date DATETIME DEFAULT NULL,
    end_date DATETIME DEFAULT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);