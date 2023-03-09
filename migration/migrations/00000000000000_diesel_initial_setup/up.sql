-- This file was automatically created by Diesel to setup helper functions
-- and other internal bookkeeping. This file is safe to edit, any future
-- changes will be added to existing projects as new migration.


-- Sets up a trigger for the given table to automatically set a column called
-- `updated_at` whenever the row is modified (unless `updated_at` was included
-- in the modified columns)
--
-- # Example
--
-- ```sql
-- CREATE TABLE users (id SERIAL PRIMARY KEY, updated_at TIMESTAMP NOT NULL DEFAULT NOW());
--
-- SELECT diesel_manage_updated_at('users');
-- ```
CREATE OR REPLACE FUNCTION diesel_manage_updated_at(_tbl regclass) RETURNS VOID AS
$$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION diesel_set_updated_at() RETURNS trigger AS
$$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto update `score_avg` for the table
--
-- 6 field are required:
-- cnt_1(bigint), cnt_2(bigint), cnt_3(bigint),
-- cnt_4(bigint), cnt_5(bigint), score_avg(float)
--
-- # Example
--
-- ```sql
-- SELECT diesel_manage_set_score_avg('t_movies');
-- ```
CREATE OR REPLACE FUNCTION diesel_manage_set_score_avg(_tbl regclass) RETURNS VOID AS
$$
BEGIN
    EXECUTE format('CREATE TRIGGER set_score_avg BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_score_avg()', _tbl);
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION diesel_set_score_avg() RETURNS trigger AS
$$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.score_avg IS NOT DISTINCT FROM OLD.score_avg
    ) THEN
        DECLARE
            total FLOAT := cast(NEW.cnt_1 + NEW.cnt_2 + NEW.cnt_3 + NEW.cnt_4 + NEW.cnt_5 AS FLOAT) + 1e-5;
        BEGIN
            NEW.score_avg := round(cast((2 * (NEW.cnt_1 / total) + 4 * (NEW.cnt_2 / total) + 6 * (NEW.cnt_3 / total) +
                                         8 * (NEW.cnt_4 / total) + 10 * (NEW.cnt_5 / total)) AS NUMERIC), 1);
        END;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


