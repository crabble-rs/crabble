# Networking Modes


## Client-Server

1. Client asks server to create a game, gets a game ID used for future communication
2. Server initially puts the game in a pending state, waiting for players
3. After at least 2 and at most 4 players have joined the game, any one player can start it.
4. Turn-Order is determined and the game starts
5. Clients can poll the current state from the server and if it's their turn submit a move
6. End game state: TODO

## Peer-to-Peer
TODO




