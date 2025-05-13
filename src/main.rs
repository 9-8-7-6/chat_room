use hyper::body;
use hyper::service::{make_service_fn, service_fn};
use hyper::{body::Body, Request, Response, Server};
use serde::Deserialize;
use serde_json;
use std::net::SocketAddr;
use tokio::sync::{mpsc, mpsc::Sender};

mod actors;
use actors::messages::MessageType;
use actors::messages::StateActorMessage;
use actors::runner::RunnerActor;
use actors::state::StateActor;

#[derive(Deserialize, Debug)]
struct IncomingBody {
    pub chat_id: i32,
    pub timestamp: i32,
    pub input: String,
    pub output: String,
}

async fn handle(
    req: Request<Body>,
    channel_sender: Sender<StateActorMessage>,
) -> Result<Response<Body>, &'static str> {
    println!("incoming message from the outside");
    let method = req.method().clone();
    println!("{}", method);
    let uri = req.uri();
    println!("{}", uri);
    let bytes = body::to_bytes(req.into_body()).await.unwrap();
    let string_body = String::from_utf8(bytes.to_vec()).expect("response was not valid utf-8");
    let value: IncomingBody = serde_json::from_str(&string_body.as_str()).unwrap();
    let message = StateActorMessage {
        message_type: MessageType::INPUT,
        chat_id: Some(value.chat_id),
        single_data: Some(format!(
            "{}>>{}>>{}>>",
            value.input, value.output, value.timestamp
        )),

        block_data: None,
    };
    channel_sender.send(message).await.unwrap();
    Ok(Response::new(format!("{:?}", value).into()))
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let (state_tx, state_rx) = mpsc::channel::<StateActorMessage>(1);
    let (runner_tx, runner_rx) = mpsc::channel::<StateActorMessage>(1);
    let channel_sender = state_tx.clone();
    tokio::spawn(async move {
        let state_actor = StateActor::new(state_rx, runner_tx);
        state_actor.run().await;
    });
    tokio::spawn(async move {
        let lib_runner_actor = RunnerActor::new(runner_rx, state_tx, 30);
        lib_runner_actor.run().await;
    });
    let server = Server::bind(&addr).serve(make_service_fn(|_conn| {
        let channel = channel_sender.clone();
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let channel = channel.clone();
                async { handle(req, channel).await }
            }))
        }
    }));
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
