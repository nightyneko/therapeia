SET search_path = public, pg_temp;

UPDATE users SET password = '$2b$12$fWwxvdX9TkWVDeopn84VbuYYMK0ixiJzoIrMNXnvkTeO2wrj0Eej6'
WHERE citizen_id = '1234567890123';

UPDATE users SET password = '$2b$12$Fc8WgF/EMe3bs3WjKw/iROhN66oUymOvL/UYljusgh372qDiyiDp.'
WHERE citizen_id = '2345678901234';

UPDATE users SET password = '$2b$12$ROjrc16338GhvKkkfAX27ufCIzf24nTLVfpzQ08fLbW1RaiNqDlQ6'
WHERE citizen_id = '3456789012345';
