extern crate clap;
extern crate chrono;
extern crate reqwest;
extern crate serde_json;
#[macro_use] extern crate prettytable;

mod connection;

use chrono::{DateTime, Duration, FixedOffset, NaiveTime};
use serde_json::Value as JsonValue;
use connection::Connection;
use prettytable::Table;
use clap::{Arg, App};

const HOST: &str = "http://transport.opendata.ch/v1";
const DEFAULT_PAGE: i32 = 0;

fn get_args() -> (String, String, i32) {
    let matches = App::new("sbb")
        .version("0.1.0")
        .author("Layle")
        .about("Fetches available connections from SBB")
        .arg(Arg::with_name("from")
            .short("f")
            .long("from")
            .required(true)
            .takes_value(true)
            .help("From"))
        .arg(Arg::with_name("to")
            .short("t")
            .long("to")
            .required(true)
            .takes_value(true)
            .help("To"))
        .arg(Arg::with_name("number")
            .short("n")
            .long("number")
            .takes_value(true)
            .help("The number/amount of connections to fetch"))
        .arg(Arg::with_name("via")
            .short("v")
            .long("via")
            .takes_value(true)
            .help("Via"))
        .get_matches();

    let from = matches.value_of("from").unwrap();
    let to = matches.value_of("to").unwrap();
    let number = matches.value_of("number").unwrap_or("1").parse::<i32>().unwrap();

    (String::from(from), String::from(to), number)
}

fn get_connections(from: &String, to: &String, limit: i32) -> JsonValue {
    let url = format!("{}/connections?from={}&to={}&page={}&limit={}", HOST, from, to, DEFAULT_PAGE, limit);

    reqwest::blocking::get(&url)
        .unwrap()
        .json::<JsonValue>()
        .unwrap()
}

fn get_field_as_string(value: &JsonValue, field_name: &str) -> String {
    let field = value.get(field_name).unwrap();
    String::from(field.as_str().unwrap_or(""))    
}

fn parse_location(connection: &JsonValue, subfield: &str) -> (String, String, String, String) {
    let field = connection.get(subfield).unwrap();
    
    let arrival = get_field_as_string(&field, "arrival");
    let departure = get_field_as_string(&field, "departure");
    let platform = get_field_as_string(&field, "platform");
    let station = field.get("station").unwrap();
    let station_name = get_field_as_string(&station, "name");
    
    (arrival, departure, platform, station_name)
}

fn parse_date(date: &str) -> DateTime::<FixedOffset> {
    DateTime::parse_from_str(&date, "%Y-%m-%dT%H:%M:%S%z").unwrap()
}

fn get_range(connections: &Vec<Connection>) -> (NaiveTime, NaiveTime) {
    let start = connections.first().unwrap().departure_date;
    let end = connections.last().unwrap().arrival_date;

    (start, end)
}

fn print_table(connections: &Vec<Connection>) {
    let (start, end) = get_range(&connections);

    let mut table = Table::new();
    table.add_row(row![b->"From", b->"Departure", b->"To", b->"Arrival", b->"Platform", b->"Duration"]);

    for connection in connections {
        let duration_fmt = format!("{}min", connection.duration.num_minutes());

        table.add_row(row![
            connection.departure_name,
            connection.departure_date,
            connection.arrival_name,
            connection.arrival_date,
            connection.platform,
            duration_fmt]);
    }

    println!("=== {} -> {} ===", start, end);
    table.printstd();
}

fn main() {
    let (from, to, number) = get_args();
    let response = get_connections(&from, &to, number);
    let connections = response.get("connections").unwrap().as_array().unwrap();

    //println!("{}", serde_json::to_string(&response).unwrap());

    for connection in connections {
        let mut connection_list = Vec::<Connection>::new();
        let sections = connection.get("sections").unwrap().as_array().unwrap();

        for section in sections {
            let (arrival, _, _, station_name_arrival) = parse_location(&section, "arrival");
            let (_, departure, platform, station_name_departure) = parse_location(&section, "departure");

            let mut duration = Duration::zero();
            let mut arrival_date: Option<DateTime<FixedOffset>> = None;
            let mut departure_date: Option<DateTime<FixedOffset>> = None;

            if !arrival.is_empty() && !departure.is_empty() {
                arrival_date = Some(parse_date(&arrival));
                departure_date = Some(parse_date(&departure));
                
                duration = arrival_date.unwrap().signed_duration_since(departure_date.unwrap());
            }

            connection_list.push(Connection {
                departure_name: station_name_departure,
                departure_date: departure_date.unwrap().time(), 
                arrival_name: station_name_arrival,
                arrival_date: arrival_date.unwrap().time(),
                duration: duration,
                platform
            });
        }

        print_table(&connection_list);
        println!();
    }
}