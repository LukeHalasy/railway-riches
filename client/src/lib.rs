use std::collections::HashMap;

use gloo_events::EventListener;
use gloo_utils::format::JsValueSerdeExt;
use js_sys::{Array, Function};
use leaflet::{
    Circle, Control, LatLng, LatLngBounds, Map, Polygon, Polyline, Rectangle, TileLayer,
};
use random_color::RandomColor;
use serde::{Deserialize, Serialize};
use store::{city, deed, rail_road, sub_city};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{console, window, Element, HtmlAnchorElement};

#[derive(Serialize, Deserialize)]
struct CircleOptions {
    radius: f32,
}

#[derive(Serialize, Deserialize)]
struct PolylineOptions {
    color: String,
}

#[derive(Serialize, Deserialize)]
struct ControlOptions {
    position: String,
}

#[derive(Serialize, Deserialize)]
struct ControlProps {
    options: ControlOptions,
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console::log_1(&"Running Leaflet example code in Rust.".into());

    let map = Map::new("map", &JsValue::NULL);
    map.setView(&LatLng::new(38.8951100, -77.0363700), 5.0);
    add_tile_layer(&map);

    for city in city::City::cities() {
        // L.circleMarker(city.latlng, {
        //   radius: 3,
        //   fillColor: 'black', // Change this to your desired fill color
        //   fillOpacity: 1,
        //   color: 'transparent' // Set the border color to transparent
        // }).addTo(map).bindPopup(city.name);

        Circle::new_with_options(
            &LatLng::new(
                city.coordinates().latitude(),
                city.coordinates().longitude(),
            ),
            &JsValue::from_serde(&CircleOptions { radius: 4000.0 })
                .expect("Unable to serialize circle options"),
        )
        .addTo(&map);
    }

    for city in sub_city::SubCity::sub_cities() {
        Circle::new_with_options(
            &LatLng::new(
                city.coordinates().latitude(),
                city.coordinates().longitude(),
            ),
            &JsValue::from_serde(&CircleOptions { radius: 4000.0 })
                .expect("Unable to serialize circle options"),
        )
        .addTo(&map);
    }

    let mut rs: HashMap<&deed::Deed, String> = HashMap::new();
    for de in deed::Deed::rails() {
        let color = RandomColor::new()
            .luminosity(random_color::Luminosity::Dark)
            .to_hex();
        rs.insert(de, color);
    }

    let d = deed::Deed::get_railroad_graph();
    for (ci, route) in d.into_iter() {
        for r in route {
            let city_two = r.0;
            Polyline::new_with_options(
                [
                    LatLng::new(ci.coordinates().latitude(), ci.coordinates().longitude()),
                    LatLng::new(
                        city_two.coordinates().latitude(),
                        city_two.coordinates().longitude(),
                    ),
                ]
                .iter()
                .map(JsValue::from)
                .collect(),
                &JsValue::from_serde(&PolylineOptions {
                    color: rs.get(&r.1).unwrap().into(),
                })
                .expect("Unable to serialize polyline options"),
            )
            .addTo(&map);
        }
    }

    // add_tile_layer(&map);
    // add_polyline(&map);
    // add_polygon(&map);
    // add_rectangle(&map);
    // add_circle(&map);
    // add_circle_with_options(&map);
    // add_control(&map);

    Ok(())
}

fn add_tile_layer(map: &Map) {
    TileLayer::new(
        "https://{s}.basemaps.cartocdn.com/light_nolabels/{z}/{x}/{y}{r}.png",
        &JsValue::NULL,
    )
    .addTo(map);
}

fn add_polyline(map: &Map) {
    Polyline::new_with_options(
        [
            LatLng::new(63.25, 11.25),
            LatLng::new(63.75, 11.75),
            LatLng::new(63.5, 12.0),
        ]
        .iter()
        .map(JsValue::from)
        .collect(),
        &JsValue::from_serde(&PolylineOptions {
            color: "red".into(),
        })
        .expect("Unable to serialize polyline options"),
    )
    .addTo(map);
}

fn add_polygon(map: &Map) {
    Polygon::new(
        [
            LatLng::new(63.25, 12.25),
            LatLng::new(63.75, 12.75),
            LatLng::new(63.5, 13.0),
        ]
        .iter()
        .map(JsValue::from)
        .collect(),
    )
    .addTo(map);
}

fn add_rectangle(map: &Map) {
    Rectangle::new(&LatLngBounds::new(
        &LatLng::new(63.25, 10.25),
        &LatLng::new(63.75, 10.75),
    ))
    .addTo(map);
}

fn add_circle(map: &Map) {
    Circle::new(&LatLng::new(63.25, 13.25)).addTo(map);
}

fn add_circle_with_options(map: &Map) {
    Circle::new_with_options(
        &LatLng::new(63.25, 13.35),
        &JsValue::from_serde(&CircleOptions { radius: 4000.0 })
            .expect("Unable to serialize circle options"),
    )
    .addTo(map);
}

fn add_control(map: &Map) {
    let props = JsValue::from_serde(&ControlProps {
        options: ControlOptions {
            position: "topleft".into(),
        },
    })
    .expect("Unable to serialize control props");

    // This callback must return a HTML div representing the control button.
    let on_add = || {
        let document = window()
            .expect("Unable to get browser window")
            .document()
            .expect("Unable to get browser document");

        let container = document
            .create_element("div")
            .expect("Unable to create div");

        container.set_class_name("leaflet-bar");

        let link = document
            .create_element("a")
            .expect("Unable to create link")
            .dyn_into::<HtmlAnchorElement>()
            .expect("Unable to cast to HtmlAnchorElement");

        link.set_href("#");
        link.set_inner_html("⬤");
        link.set_title("Create a new foobar.");

        let on_click = EventListener::new(&link, "click", |_| {
            console::log_1(&"Control button click.".into());
        });

        on_click.forget();

        container
            .append_child(&link)
            .expect("Unable to add child element");

        container
    };

    let on_add_closure = Closure::wrap(Box::new(on_add) as Box<dyn FnMut() -> Element>);

    js_sys::Reflect::set(&props, &JsValue::from("onAdd"), on_add_closure.as_ref())
        .expect("Unable to set onAdd()");

    on_add_closure.forget();

    let control_class = Control::extend(&props)
        .dyn_into::<Function>()
        .expect("Unable to cast to Function");

    let control_button: Control = JsCast::unchecked_into(
        js_sys::Reflect::construct(&control_class, &Array::new())
            .expect("Unable to run constructor"),
    );

    control_button.addTo(map);
}
