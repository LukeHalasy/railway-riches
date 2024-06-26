use futures::channel::mpsc::Sender;
use leptos::*;
use leptos_meta::Title;
use leptos_router::*;
use store::{ClientMessage, Event};

use crate::{app::PlayerId, pre_game::layout::Layout};

#[derive(Params, PartialEq)]
pub struct LobbyParams {
    pub id: usize,
}

#[component]
pub fn Lobby() -> impl IntoView {
    let lobby_id = leptos_router::use_params::<LobbyParams>()
        .with(|params| params.as_ref().map(|params| params.id).unwrap_or_default());

    let game_state =
        use_context::<ReadSignal<Option<store::State>>>().expect("Expected a game state signal");
    let player_id = use_context::<PlayerId>().expect("Expected a player id signal");
    let tx = use_context::<Sender<ClientMessage>>().expect("Expected the tx sender");

    view! {
        <Title text={move || format!("Lobby {}", lobby_id)} />
        <Layout>
            <h1>{move || format!("Lobby {}", lobby_id)}</h1>
            <ul>
                <For
                    each=move || game_state.get().unwrap().players
                    key=|counter| counter.0
                    children=move |(id, player)| {
                        let mut curr_id = id.to_string();
                        if game_state.get().unwrap().game_host == Some(id) {
                            curr_id += " (host)";
                        }

                        if player.computer {
                            curr_id += " (computer)";
                        }

                        if id == player_id.0.get().unwrap() {
                            view! {
                                <li>
                                    <strong>{ curr_id }</strong>
                                </li>
                            }
                        } else {
                            view! {
                                <li>
                                    { curr_id }
                                </li>
                            }
                        }
                    }
                />
            </ul>
            {
                if game_state.get().unwrap().game_host == Some(player_id.0.get().unwrap()) {
                    view! {
                        // add a button for the host to add computers to the game
                        <input type="submit" value="Add Computer" on:click={
                            let mut tx = tx.clone();
                            move |_| {
                                let _ = tx
                                    .try_send(ClientMessage::AddComputer(lobby_id.try_into().unwrap()));
                            }
                        }/>
                        <input type="submit" value="start game" on:click={
                            let mut tx = tx.clone();
                            move |_| {
                                let _ = tx
                                    .try_send(ClientMessage::Event(Event::Start {
                                        player_id: player_id.0.get().unwrap(),
                                    }));
                            }
                        }/>
                    }.into_view()
                } else {
                    view! {
                        <p>Waiting for host to start game</p>
                    }.into_view()
                }
            }
        </Layout>
    }
}
