-- === Trigger Test Suite ===

-- === CASE: Create simple trigger ===
-- EXPECT: success
CREATE TRIGGER update_timestamp UPDATE ON users
BEGIN
  UPDATE users SET email = 'triggered_' || email WHERE id = NEW.id;
END;

-- === CASE: Create BEFORE INSERT trigger ===
-- EXPECT: success
CREATE TRIGGER before_insert_user BEFORE INSERT ON users
BEGIN
  SELECT CASE WHEN NEW.name IS NULL THEN RAISE(ABORT, 'Name required') END;
END;

-- === CASE: Create AFTER DELETE trigger ===
-- EXPECT: success
CREATE TRIGGER after_delete_user AFTER DELETE ON users
BEGIN
  INSERT INTO audit_log (action, table_name, timestamp) VALUES ('delete', 'users', datetime('now'));
END;

-- === CASE: Create INSTEAD OF trigger (for views) ===
-- EXPECT: success
CREATE TRIGGER instead_of_update INSTEAD OF UPDATE ON user_view
BEGIN
  UPDATE users SET name = NEW.name WHERE id = OLD.id;
END;

-- === CASE: Create trigger with WHEN clause ===
-- EXPECT: success
CREATE TRIGGER high_value_order AFTER INSERT ON orders
WHEN NEW.total > 1000
BEGIN
  INSERT INTO alerts (message) VALUES ('High value order: ' || NEW.total);
END;

-- === CASE: Create trigger with multiple statements ===
-- EXPECT: success
CREATE TRIGGER multi_action AFTER UPDATE ON users
BEGIN
  INSERT INTO audit_log (action, table_name, old_id) VALUES ('update', 'users', OLD.id);
  INSERT INTO audit_log (action, table_name, new_id) VALUES ('update', 'users', NEW.id);
END;

-- === CASE: Drop trigger ===
-- EXPECT: success
DROP TRIGGER update_timestamp;

-- === CASE: Drop trigger if exists ===
-- EXPECT: success
DROP TRIGGER IF EXISTS nonexistent_trigger;

-- === CASE: Create trigger with NEW and OLD ===
-- EXPECT: success
CREATE TRIGGER log_changes UPDATE ON users
FOR EACH ROW
BEGIN
  INSERT INTO audit_log (action, old_value, new_value) VALUES ('update', OLD.email, NEW.email);
END;

-- === CASE: Create trigger with conditional ===
-- EXPECT: success
CREATE TRIGGER conditional_update AFTER UPDATE ON users
WHEN OLD.name != NEW.name
BEGIN
  INSERT INTO audit_log (action, old_name, new_name) VALUES ('name_change', OLD.name, NEW.name);
END;

-- === CASE: Create trigger for INSERT with NEW ===
-- EXPECT: success
CREATE TRIGGER log_insert AFTER INSERT ON users
FOR EACH ROW
BEGIN
  INSERT INTO audit_log (action, new_id, new_name) VALUES ('insert', NEW.id, NEW.name);
END;

-- === CASE: Create trigger for DELETE with OLD ===
-- EXPECT: success
CREATE TRIGGER log_delete AFTER DELETE ON users
FOR EACH ROW
BEGIN
  INSERT INTO audit_log (action, old_id, old_name) VALUES ('delete', OLD.id, OLD.name);
END;

-- === CASE: Create trigger with RAISE ===
-- EXPECT: success
CREATE TRIGGER prevent_delete BEFORE DELETE ON users
BEGIN
  SELECT CASE WHEN OLD.id < 10 THEN RAISE(ABORT, 'Cannot delete protected users') END;
END;
