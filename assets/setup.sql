DO $SCRIPT$
DECLARE
    aid integer;
BEGIN
    INSERT INTO accounts 
        (username, password) 
    VALUES 
        ('USERNAME', '<PASSWORD>')
    RETURNING "id" into aid;

    INSERT INTO characters
        (name, account_id, x, y)
    VALUES
        ('NAME', aid, 0, 0);
END
$SCRIPT$;