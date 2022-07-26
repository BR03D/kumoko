# Kumoko

## Problems for the Now
- The docs **really** are not great.
- Events that arent Messages are not tested. Like at all.
- There are no examples for splitting the server/client. Its not really tested a lot either.
- text 

## Problems for the FarFuture<sup>tm</sup>

## Fixed
- Dropping the Server **will** make the code panic almost immediatly. Thats not good.
- instance::Receivers are initialized when a connection happens, and will only drop themselves when trying to send an Event. If Clients connect and never disconnect it will simply never drop - leaking memory in the process. Maybe add a timeout?