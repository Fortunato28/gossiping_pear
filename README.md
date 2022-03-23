# Gossiping_peer

Test task completed according to [this exercise](https://hackmd.io/@r3XngjBBSumx2rU-hKU7Qg/BkbHS80cv). Despinte on the task, peers will sending not a random messages, but their period arguments for more clarity.
# TODO
 - [ ] Testing. Probably use some DI to intersept output
 - [ ] CI
 - [ ] Add ability to handle not only text messages

# Example

1. Run the first peer which is not connecting anywhere
```
./gossiping_peer --period=1 --port=8080
```
2. Run the second peer which is connecting to previous one
```
./gossiping_peer --period=2 --port=8081 --connection="127.0.0.1:8080" 
```
3. Run the third peer which is connecting to the first one, but also will gossip with the second because of address sharing inside network
```
./gossiping_peer --period=3 --port=8082 --connection="127.0.0.1:8080"  
```

## Example output

Third peer will receive something like this:
```
Listening on: 127.0.0.1:8082
WebSocket handshake has been successfully completed
Received a message from 127.0.0.1:8080: "1"
WebSocket handshake has been successfully completed
Received a message from 127.0.0.1:8081: "2"
Received a message from 127.0.0.1:8080: "1"
Received a message from 127.0.0.1:8080: "1"
Received a message from 127.0.0.1:8081: "2"
Received a message from 127.0.0.1:8080: "1"
```
