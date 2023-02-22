ALTER TABLE weights RENAME TO tmp_weights;
CREATE TABLE weights (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    weight DOUBLE NOT NULL,
    measurement_date DATE NOT NULL UNIQUE
);
INSERT INTO weights (id, weight, measurement_date)
SELECT id, weight_value, measurement_date FROM tmp_weights;
DROP TABLE tmp_weights;