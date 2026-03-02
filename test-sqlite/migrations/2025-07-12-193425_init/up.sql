CREATE TABLE pokemons (
id INTEGER NOT NULL PRIMARY KEY autoincrement,
name TEXT NOT NULL,
next_evolution_id INTEGER,
prev_evolution_id INTEGER,
description TEXT NOT NULL,
image_data_id INTEGER NOT NULL,
base_stats_id INTEGER NOT NULL,
FOREIGN KEY (next_evolution_id) REFERENCES pokemons (id),
FOREIGN KEY (prev_evolution_id) REFERENCES pokemons (id),
FOREIGN KEY (image_data_id) REFERENCES image_data (id),
FOREIGN KEY (base_stats_id) REFERENCES base_stats (id)
) ;

CREATE TABLE base_stats (
id INTEGER NOT NULL PRIMARY KEY autoincrement,
hp INTEGER NOT NULL,
attack INTEGER NOT NULL,
defense INTEGER NOT NULL,
special_attack INTEGER NOT NULL,
special_defense INTEGER NOT NULL,
speed INTEGER NOT NULL,
pokemon_id INTEGER NOT NULL,
FOREIGN KEY (pokemon_id) REFERENCES pokemons (id)
) ;

CREATE TABLE image_data (
id INTEGER NOT NULL PRIMARY KEY autoincrement,
sprite TEXT NOT NULL,
thumbnail TEXT NOT NULL,
hires TEXT NOT NULL,
pokemon_id INTEGER NOT NULL,
FOREIGN KEY (pokemon_id) REFERENCES pokemons (id)
) ;

CREATE TABLE types (
id INTEGER NOT NULL PRIMARY KEY autoincrement,
name TEXT NOT NULL
) ;

CREATE TABLE pokemon_types (
pokemon_id INTEGER NOT NULL,
type_id INTEGER NOT NULL,
FOREIGN KEY (pokemon_id) REFERENCES pokemons (id),
FOREIGN KEY (type_id) REFERENCES types (id),
PRIMARY KEY (pokemon_id, type_id)
) ;
