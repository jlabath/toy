use csv::Writer;
use std::env;
use std::fs::File;
use std::io::{stdout, BufReader, BufWriter};
use tokio::sync::mpsc;
use tokio::task;

mod account;
mod engine;
mod error;
mod transaction;

#[tokio::main]
async fn main() -> Result<(), error::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(error::Error::Custom(
            "input transactions file argument is missing",
        ));
    }

    //start the engine
    let (engine_handle, channel) = engine::start().await;

    //run the producer in its own thread so as not to block the runtime
    let producer =
        task::spawn_blocking(move || read_and_push_transactions(&args[1], channel)).await;
    match producer {
        Err(e) => eprintln!("Producer task error: {}", e),
        Ok(Err(e)) => eprintln!("Producer completed with error: {}", e),
        Ok(Ok(())) => (),
    }

    match engine_handle.await {
        Err(_) => Err(error::Error::Custom("Engine task error")),
        Ok(accounts) => {
            //if this wasn't last action of the program this would be sent to a separate thread/task
            //but it's ok here since we are done
            let mut wtr = Writer::from_writer(BufWriter::new(stdout()));
            for (_, account) in accounts.iter() {
                wtr.serialize(account)?;
            }
            wtr.flush()?;
            Ok(())
        }
    }
}

/// Reads the CSV file and push Transaction into the provided channel.
fn read_and_push_transactions(
    fname: &str,
    ch: mpsc::Sender<transaction::Transaction>,
) -> Result<(), error::Error> {
    //open input file
    let f = File::open(fname)?;

    let br = BufReader::new(f);
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(br);

    for record in reader.deserialize() {
        match record {
            Ok(tx) => {
                ch.blocking_send(tx)?;
            }
            Err(e) => {
                //don't abort reading on one invalid input but log it
                eprintln!("problem reading transaction: {}", e);
            }
        }
    }

    Ok(())
}
