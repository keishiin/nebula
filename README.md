# nebula

## Requirements
```
Rust 1.77.1
Docker -> Running postgres
```

## start postgres in docker
``` 
docker run -p 5432:5432 --name some-postgres -e POSTGRES_PASSWORD=mysecretpassword -d postgres
```

## Make the .env file
```
touch .env
```

## Open the env file and add the following fields
```
DISCORD_TOKEN=your_bot_token
DATABASE_URL=the_url_to_the_db_in_docker
```

## Start the bot
```
cargo run
```
