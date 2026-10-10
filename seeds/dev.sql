insert into users (id, email, password_hash, role)
values (
    uuidv7(),
    'admin@example.com',
    'hashed::password',
    'admin'
)
on conflict (email) do nothing;
