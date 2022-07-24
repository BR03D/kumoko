# Kumoko

### Problems for the Now
- [ ] dropping the Server **will** make the code panic almost immediatly.
Thats not good.

- [x] text

## Problems for the FarFuture<sup>tm</sup>
instance::Receivers are initialized when a connection happens, and 
are not dropped if the server drops.cThey will drop themselves when
trying to send an Event and the server::Receiver is dropped - however
if you drop the server and you have connected clients who never send
any messages the instance::Receiver never drops. This is very difficult
to fix. Does anyone *really* want this fixed?