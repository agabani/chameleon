create table public."user"
(
    id        bigserial
        constraint user_pk
            primary key,
    public_id uuid not null,
    name      text not null
);

create unique index user_public_id_uindex
    on public."user" (public_id);

create table public.local
(
    id        bigserial
        constraint local_pk
            primary key,
    public_id uuid   not null,
    user_id   bigint not null
        constraint local_user_id_fk
            references public."user"
            on delete cascade
);

create unique index local_public_id_uindex
    on public.local (public_id);

create index local_user_id_index
    on public.local (user_id);

create table public.lobby
(
    id        bigserial
        constraint lobby_pk
            primary key,
    public_id uuid not null,
    name      text not null
);

create unique index lobby_public_id_uindex
    on public.lobby (public_id);

create table public.lobby_member
(
    id       bigserial
        constraint lobby_member_pk
            primary key,
    lobby_id bigint  not null
        constraint lobby_player_lobby_id_fk
            references public.lobby
            on delete cascade,
    user_id  bigint  not null,
    host     boolean not null
);

create index lobby_member_lobby_id_index
    on public.lobby_member (lobby_id);

create unique index lobby_member_user_id_lobby_id_uindex
    on public.lobby_member (user_id, lobby_id);

create index lobby_member_user_id_index
    on public.lobby_member (user_id);

