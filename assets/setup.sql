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
        (name, account_id)
    VALUES
        ('NAME', aid);
END
$SCRIPT$;