use std::collections::HashMap;
use std::fmt::format;
use rustengan::*;
use std::io::{StdoutLock, Write};
use serde::{Deserialize, Serialize};
use anyhow::{bail, Context};
use crate::Payload::{BroadcastOk, ReadOk, TopologyOk};


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Broadcast{message:usize},
    BroadcastOk,
    Read,
    ReadOk{messages:Vec<usize>},
    Topology{
        topology:HashMap<String, Vec<String>>
    },
    TopologyOk,
}

struct BroadcastNode {
    node: String,
    id: usize,
    messages: Vec<usize>,
}

impl Node<(), Payload> for BroadcastNode {
    fn from_init(_: (), init: rustengan::Init) -> anyhow::Result<Self> {
        Ok(Self {
            node:init.node_id,
            id:1,
            messages:Vec::new()
        })
    }

    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let mut reply = input.into_reply(Some(&mut self.id));
        match reply.body.payload {
            Payload::Broadcast{..}  => {
                reply.body.payload = Payload::BroadcastOk;
                serde_json::to_writer(&mut *output, &reply).context("serialize response to echo")?;
                output.write_all(b"\n").context("write trailing newline")?;
                self.id += 1;
            }
            Payload::Read=> {
                reply.body.payload = ReadOk{messages:self.messages.clone()} ;
                serde_json::to_writer(&mut *output, &reply).context("serialize response to echo")?;
                output.write_all(b"\n").context("write trailing newline")?;
                self.id += 1;
            },
            Payload::Topology {..} => {
                reply.body.payload = Payload::TopologyOk;
                serde_json::to_writer(&mut *output, &reply).context("serialize response to echo")?;
                output.write_all(b"\n").context("write trailing newline")?;
                self.id += 1;
            },
            BroadcastOk| ReadOk{..}| TopologyOk =>{}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, BroadcastNode, _>(())
}
