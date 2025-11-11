update projects
set
    system_created = true
where
    tenant = 'system'
    and id = 'system';