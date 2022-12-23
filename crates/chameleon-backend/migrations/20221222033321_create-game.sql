create table public.game
(
    id        bigserial
        constraint game_pk
            primary key,
    public_id uuid not null,
    name      text not null
);

create unique index game_public_id_uindex
    on public.game (public_id);

create table public.game_player
(
    id      bigserial
        constraint game_player_pk
            primary key,
    game_id bigint  not null
        constraint game_player_game_id_fk
            references public.game
            on delete cascade,
    user_id bigint  not null,
    host    bool    not null
);

create index game_player_game_id_index
    on public.game_player (game_id);

create unique index game_player_user_id_game_id_uindex
    on public.game_player (user_id, game_id);

create index game_player_user_id_index
    on public.game_player (user_id);

