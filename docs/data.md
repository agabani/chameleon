# Data

user (user)

| column    | type   | reference |
| --------- | ------ | --------- |
| id        | bigint |           |
| public_id | uuid   |           |

local (user authentication)

| column    | type   | reference        |
| --------- | ------ | ---------------- |
| id        | bigint |                  |
| public_id | uuid   |                  |
| user_id   | bigint | user:id (delete) |

session (user session)

| column    | type   | reference        |
| --------- | ------ | ---------------- |
| id        | bigint |                  |
| public_id | uuid   |                  |
| user_id   | bigint | user:id (delete) |

game

| column    | type   | reference |
| --------- | ------ | --------- |
| id        | bigint |           |
| public_id | uuid   |           |

game_player

| column     | type   | reference            |
| ---------- | ------ | -------------------- |
| id         | bigint |                      |
| game_id    | bigint | game:id (delete)     |
| session_id | bigint | session:id (nothing) |
