insert into users (id, email, password_hash, role)
values (
    gen_random_uuid(),
    'admin@example.com',
    'hashed::password',
    'admin'
)
on conflict (email) do nothing;
