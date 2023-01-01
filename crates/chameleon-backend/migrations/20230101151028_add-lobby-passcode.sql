alter table public.lobby
    add passcode text;

alter table public.lobby
    add require_passcode bool default FALSE not null;

