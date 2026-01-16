/*load extension*/
load postgres;

/*create secret*/
CREATE PERSISTENT SECRET if not exists {db_name}_secret (
    TYPE POSTGRES,
    HOST '{host}',
    PORT {port},
    DATABASE {db_name},
    USER '{user}',
    PASSWORD '{password}'
);

ATTACH if not exists '' AS {db_name} (TYPE POSTGRES, SECRET {db_name}_secret);