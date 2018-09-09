#[macro_use]
extern crate actix;
extern crate actix_web;
#[macro_use]
extern crate serde_derive;

use actix::*;
use actix_web::{fs, http, server, ws, App, Form, HttpResponse, State};
use std::collections::HashSet;

#[derive(Message, Deserialize, Debug, Copy, Clone)]
struct Command {
    state: i16,
}

#[derive(Message)]
struct Connect {
    addr: Recipient<Command>,
}

#[derive(Message)]
struct Disconnect {
    addr: Recipient<Command>,
}

#[derive(Default)]
struct Orchestrator {
    tomus: HashSet<Recipient<Command>>,
}

struct AppState {
    orchestrator: Addr<Orchestrator>,
}

struct Ws;

impl Actor for Orchestrator {
    type Context = Context<Self>;
}

impl Handler<Command> for Orchestrator {
    type Result = ();

    fn handle(&mut self, msg: Command, _ctx: &mut Context<Self>) -> Self::Result {
        println!("Sending command to {} Tomus", self.tomus.len());
        for tomu in &self.tomus {
            tomu.do_send(msg);
        }
    }
}

impl Handler<Connect> for Orchestrator {
    type Result = ();

    fn handle(&mut self, msg: Connect, _ctx: &mut Context<Self>) -> Self::Result {
        self.tomus.insert(msg.addr);
    }
}

impl Handler<Disconnect> for Orchestrator {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Context<Self>) -> Self::Result {
        self.tomus.remove(&msg.addr);
    }
}

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self, AppState>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Got a Tomu!");
        ctx.state().orchestrator.do_send(Connect {
            addr: ctx.address().recipient(),
        });
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        println!("Tomu disconnected.");
        ctx.state().orchestrator.do_send(Disconnect {
            addr: ctx.address().recipient(),
        });
        Running::Stop
    }
}

impl Handler<Command> for Ws {
    type Result = ();

    fn handle(&mut self, msg: Command, ctx: &mut Self::Context) {
        ctx.text(format!("{}", msg.state));
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for Ws {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => println!("WS message: {}", text),
            ws::Message::Close(_) => ctx.stop(),
            _ => (),
        }
    }
}

fn set_state((state, form): (State<AppState>, Form<Command>)) -> HttpResponse {
    let command = form.into_inner();
    println!("Setting: {:?}", command);
    state.orchestrator.do_send(command);
    HttpResponse::Ok().body("")
}

fn main() {
    let sys = actix::System::new("tomu-web-remote");
    let orchestrator = Arbiter::start(|_| Orchestrator::default());

    server::new(move || {
        App::with_state(AppState {
            orchestrator: orchestrator.clone(),
        }).handler("/static/", fs::StaticFiles::new("static/").unwrap())
            .resource("/", |r| r.f(|_req| fs::NamedFile::open("index.html")))
            .resource("/ws-tomu", |r| r.f(|req| ws::start(req, Ws)))
            .resource("/set", |r| r.method(http::Method::POST).with(set_state))
    }).bind("[::1]:8080")
        .unwrap()
        .start();

    sys.run();
}
