use futures::{SinkExt, StreamExt};

use leptos_meta::{provide_meta_context, Stylesheet, Title};
use reqwasm::websocket::{futures::WebSocket, Message};

use leptos::*;

use leptos_router::{Route, Router, Routes};

use store::{ClientMessage, Event, ServerMessage, State};

use web_sys::console;
// use server::ServerMessage;

use crate::game::game::Game;
use crate::pre_game::home::Home;
use crate::pre_game::join::Join;
use crate::pre_game::lobby::{Lobby, LobbyParams};
use crate::pre_game::rules::Rules;
use gloo_timers::future::TimeoutFuture;

pub type Error = String;

#[derive(Copy, Clone, Debug)]
pub struct PlayerId(pub ReadSignal<Option<store::PlayerId>>);

#[component]
pub fn App() -> impl IntoView {
    let ws = WebSocket::open("ws://0.0.0.0:8000").unwrap();
    let (mut write, mut read) = ws.split();

    let (in_tx, mut in_rx) = futures::channel::mpsc::channel::<ClientMessage>(1000);
    provide_context(in_tx);

    // TODO: Uncomment
    let (state, set_state) = create_signal(None::<State>);
    provide_context(state);
    provide_context(set_state);

    let (player_id, set_player_id) = create_signal(None::<store::PlayerId>);
    provide_context(PlayerId(player_id));

    let (error, set_error) = create_signal(None::<Error>);
    provide_context(error);

    leptos::spawn_local(async move {
        while let Some(msg) = in_rx.next().await {
            // log::debug!("got event from channel! {}", s);
            write
                .send(Message::Bytes(bincode::serialize(&msg).unwrap()))
                .await
                .unwrap();
        }
    });

    leptos::spawn_local(async move {
        // wait a second
        while let Some(msg) = read.next().await {
            if let Message::Bytes(bytes) = msg.unwrap() {
                console::log_1(&"got message from server!".to_string().into());
                if let Ok(server_message) = bincode::deserialize::<ServerMessage>(&bytes) {
                    match server_message {
                        ServerMessage::Event(event) => {
                            web_sys::console::log_1(
                                &format!("got event from server! {:?}", event).into(),
                            );
                            set_state.update(|state| {
                                let state = state.as_mut().unwrap();

                                state.consume(&event);
                            });

                            // we should only wait if the game stage is start
                            if let store::Stage::InGame(_stage) =
                                state.get().as_ref().unwrap().stage
                            {
                                web_sys::console::log_1(&"waiting 1 second...".to_string().into());
                                TimeoutFuture::new(1_000).await;
                            }

                            if let Event::Start { player_id: _ } = event {
                                // get the lobby id from the url
                                let lobby_id =
                                    leptos_router::use_params::<LobbyParams>().with(|params| {
                                        params.as_ref().map(|params| params.id).unwrap_or_default()
                                    });

                                // navigate to the game
                                let navigate = leptos_router::use_navigate();
                                navigate(&format!("/game/{}", lobby_id), Default::default())
                            }
                        }
                        ServerMessage::Error(error) => {
                            web_sys::console::error_1(
                                &format!("got error from server! {:?}", error).into(),
                            );

                            set_error.update(|e| *e = Some(error));
                        }
                        ServerMessage::Connection(player_id) => {
                            web_sys::console::log_1(
                                &format!("got connection from server! {:?}", player_id).into(),
                            );
                            set_player_id.update(|id| {
                                *id = Some(player_id);
                            });
                            web_sys::console::log_1(
                                &format!("got player id from server! {:?}", player_id).into(),
                            );
                        }
                        ServerMessage::GameCreated(game_id) => {
                            web_sys::console::log_1(
                                &format!("got game created from server! {:?}", game_id).into(),
                            );

                            // initialize the game state
                            set_state.update(|state| {
                                *state = Some(State::default());
                            });

                            // navigate to the lobby
                            let navigate = leptos_router::use_navigate();
                            navigate(&format!("/lobby/{}", game_id), Default::default());
                        }
                        ServerMessage::GameJoined(game_id) => {
                            web_sys::console::log_1(
                                &format!("succesfully joined game! {:?}", game_id).into(),
                            );

                            // initialize the game state
                            set_state.update(|state| {
                                *state = Some(State::default());
                            });

                            // navigate to the lobby
                            let navigate = leptos_router::use_navigate();
                            navigate(&format!("/lobby/{}", game_id), Default::default());
                        }
                    }
                }
            }
        }
    });

    provide_meta_context();
    view! {
        <Stylesheet id="leptos" href="/src/output.css"/>
        <Title formatter=|text| format!("Railway Riches - {text}")/>
        <Router>
            <Routes>
                <Route path="/" view=Home />
                <Route path="/join" view=Join />
                <Route path="/rules" view=Rules />
                <Route path="/lobby/:id" view=Lobby />
                <Route path="/game/:id" view=Game />
            </Routes>
        </Router>
    }
}
