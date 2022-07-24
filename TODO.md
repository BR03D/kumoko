# Kumoko

## Problems for the Now
- dropping the Server **will** make the code panic almost immediatly. Thats not good.
- The docs **really** are not great.
- Events that arent Requests are not tested. Like at all.
- There are no examples for splitting the server/client. Its not really tested a lot either

## Problems for the FarFuture<sup>tm</sup>
- instance::Receivers are initialized when a connection happens, and will only drop themselves when trying to send an Event. If Clients connect and never disconnect it will simply never drop - leaking memory in the process. Maybe add a timeout?