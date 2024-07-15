use actix::prelude::*;

// Define the Ping message
struct Ping;
impl Message for Ping {
    type Result = ();
}

// Define the Pong message
struct Pong;
impl Message for Pong {
    type Result = ();
}

// Define the PingActor
struct PingActor {
    counter: usize,
    pong: Option<Addr<PongActor>>,
}

impl Actor for PingActor {
    type Context = Context<Self>;
}

impl Handler<Pong> for PingActor {
    type Result = ();

    fn handle(&mut self, _msg: Pong, ctx: &mut Context<Self>) {
        if let Some(pong_addr) = &self.pong {
            self.counter += 1;
            if self.counter < 10 {
                println!("Ping received Pong, counter: {}", self.counter);
                pong_addr.do_send(Ping);
            } else {
                // Could stop here but wouldn't be able to receive messages.
                // println!("Ping stops");
                // ctx.stop();
            }
        }
    }
}

// Define a message to set the Ping reference
struct SetPong(Addr<PongActor>);
impl Message for SetPong {
    type Result = ();
}

impl Handler<SetPong> for PingActor {
    type Result = ();

    fn handle(&mut self, msg: SetPong, _ctx: &mut Context<Self>) {
        let pong_addr = msg.0;
        self.pong = Some(pong_addr.clone());
        pong_addr.do_send(Ping);
    }
}

// Define the PongActor
struct PongActor {
    ping: Addr<PingActor>,
}

impl Actor for PongActor {
    type Context = Context<Self>;
}

impl Handler<Ping> for PongActor {
    type Result = ();

    fn handle(&mut self, _msg: Ping, _ctx: &mut Context<Self>) {
        println!("Pong received Ping");
        self.ping.do_send(Pong);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn ping_pong_works() {
        let ping_addr = PingActor {
            counter: 0,
            pong: None,
        }
        .start();

        let pong_addr = PongActor {
            ping: ping_addr.clone(),
        }
        .start();

        // Set the references between PingActor and PongActor
        ping_addr.send(SetPong(pong_addr.clone())).await.unwrap();

        // Wait for the actors to process the messages
        actix_rt::time::sleep(std::time::Duration::from_millis(1)).await;

        let counter = ping_addr.send(GetCounter).await.unwrap();
        assert_eq!(counter, 10);

        // Define a message to retrieve the actor's counter
        #[derive(Message)]
        #[rtype(result = "usize")]
        struct GetCounter;

        impl Handler<GetCounter> for PingActor {
            type Result = usize;

            fn handle(&mut self, _msg: GetCounter, _: &mut Context<Self>) -> Self::Result {
                self.counter
            }
        }
    }
}
