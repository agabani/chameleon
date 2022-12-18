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
