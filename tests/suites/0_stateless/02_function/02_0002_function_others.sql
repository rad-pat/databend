SELECT '=== IGNORE ===';
select count() from numbers(100) where ignore(number + 1);
select count() from numbers(100) where not ignore(toString(number + 3), 1, 4343, 4343, 'a');
SELECT '=== INET_ATON ===';
SELECT INET_ATON('10.0.5.9');
SELECT INET_ATON(NULL);
SELECT INET_ATON('hello');
SELECT '=== INET_NTOA ===';
SELECT INET_NTOA(167773449);
SELECT INET_NTOA('167773449');
SELECT INET_NTOA('0');
SELECT INET_NTOA('hello');

