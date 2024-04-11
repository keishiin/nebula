use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use poise::CreateReply;
use reqwest;
use serde::Deserialize;
use serenity::Colour;
use serenity::CreateEmbed;

const POKEURL: &str = "https://pokeapi.co/api/v2/pokemon/";

#[derive(Deserialize)]
struct Pokemon {
    name: String,
    types: Vec<PokemonType>,
    sprites: Sprites,
}

#[derive(Deserialize)]
struct PokemonType {
    #[serde(rename = "type")]
    pokemon_type: Type,
}

#[derive(Deserialize)]
struct Type {
    name: String,
}

#[derive(Deserialize)]
struct Sprites {
    front_default: Option<String>,
}

#[poise::command(slash_command)]
pub async fn get_random_pokemon(ctx: Context<'_>, num: u64) -> Result<(), Error> {
    let url = format!("{}/{}", POKEURL, num);
    let pokemon = reqwest::get(url).await?.json::<Pokemon>().await?;
    println!("{}", pokemon.name);

    let mut description = "Types: ".to_string();
    for poki_type in &pokemon.types {
        description.push_str(&format!("{} ", poki_type.pokemon_type.name));
    }

    let embed = CreateEmbed::new()
        .title(format!("{}'s", pokemon.name))
        .image(pokemon.sprites.front_default.unwrap_or("".to_string()))
        .description(description)
        .color(Colour::BLUE);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn get_pokemon_by_name(ctx: Context<'_>, name: String) -> Result<(), Error> {
    let url = format!("{}/{}", POKEURL, name);
    let pokemon = reqwest::get(url).await?.json::<Pokemon>().await?;
    println!("{}", pokemon.name);

    let mut description = "Types: ".to_string();
    for poki_type in &pokemon.types {
        description.push_str(&format!("{} ", poki_type.pokemon_type.name));
    }

    let embed = CreateEmbed::new()
        .title(format!("{}'s", name))
        .image(pokemon.sprites.front_default.unwrap_or("".to_string()))
        .description(description)
        .color(Colour::BLUE);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}
