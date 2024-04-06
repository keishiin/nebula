# nebula



## start postgres in docker
``` 
docker run -p 5432:5432 --name some-postgres -e POSTGRES_PASSWORD=mysecretpassword -d postgres
```
## create table 
```
CREATE TABLE IF NOT EXISTS public.leveling
(
    user_id character varying COLLATE pg_catalog."default" NOT NULL,
    guild_id character varying COLLATE pg_catalog."default" NOT NULL,
    level bigint,
    curr_exp bigint,
    next_lvl bigint,
    CONSTRAINT leveling_pkey PRIMARY KEY (user_id)
)
```

