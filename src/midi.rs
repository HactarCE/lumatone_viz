use std::sync::mpsc;

use midir::{
    MidiInput, MidiInputConnection, MidiInputPort, MidiOutput, MidiOutputConnection, MidiOutputPort,
};
use midly::live::LiveEvent;

pub enum MidiState {
    Uninit(Option<eyre::Report>),
    Ready {
        state: ReadyMidiState,
        input_port: Option<MidiInputPort>,
        output_port: Option<MidiOutputPort>,
    },
    Connected(ConnectedMidiState),
}

impl Default for MidiState {
    fn default() -> Self {
        Self::Uninit(None)
    }
}

impl From<eyre::Report> for MidiState {
    fn from(e: eyre::Report) -> Self {
        Self::Uninit(Some(e))
    }
}

impl MidiState {
    pub fn uninit(&mut self) {
        *self = Self::default();
    }

    pub fn init(&mut self) {
        match ReadyMidiState::new() {
            Ok(state) => {
                *self = Self::Ready {
                    state,
                    input_port: None,
                    output_port: None,
                }
            }
            Err(e) => *self = e.into(),
        }
    }
}

pub struct ReadyMidiState {
    input: MidiInput,
    output: MidiOutput,
}

impl ReadyMidiState {
    pub fn new() -> eyre::Result<Self> {
        let input = MidiInput::new("lumatone_viz_output")?;
        let output = MidiOutput::new("lumatone_viz_output")?;
        Ok(Self { input, output })
    }

    pub fn input_ports(&self) -> Vec<(MidiInputPort, String)> {
        let default_name = |_| "<error>".to_string();
        self.input
            .ports()
            .into_iter()
            .map(|port| {
                let name = self.input.port_name(&port).unwrap_or_else(default_name);
                (port, name)
            })
            .collect()
    }

    pub fn output_ports(&self) -> Vec<(MidiOutputPort, String)> {
        let default_name = |_| "<error>".to_string();
        self.output
            .ports()
            .into_iter()
            .map(|port| {
                let name = self.output.port_name(&port).unwrap_or_else(default_name);
                (port, name)
            })
            .collect()
    }

    pub fn connect(
        self,
        input_port: MidiInputPort,
        output_port: MidiOutputPort,
    ) -> eyre::Result<ConnectedMidiState> {
        let default_name = |_| "<error>".to_string();
        let input_port_name = self
            .input
            .port_name(&input_port)
            .unwrap_or_else(default_name);
        let output_port_name = self
            .output
            .port_name(&output_port)
            .unwrap_or_else(default_name);

        let output_connection = self
            .output
            .connect(&output_port, "lumatone_viz_input_port")?;

        let (event_tx, event_rx) = mpsc::channel();

        let state = MidiPassthroughListenerState {
            output_connection,
            event_tx,
        };

        let input_connection = self.input.connect(
            &input_port,
            "lumatone_viz_input_port",
            |_timestamp, message, state| {
                _ = state.output_connection.send(message);
                match midly::live::LiveEvent::parse(message) {
                    Ok(event) => _ = state.event_tx.send(event.to_static()),
                    Err(e) => eprintln!("error parsing MIDI message {message:x?}: {e}"),
                }
            },
            state,
        )?;

        Ok(ConnectedMidiState {
            input_port: (input_port, input_port_name),
            output_port: (output_port, output_port_name),
            input_connection,
            event_rx,
        })
    }
}

pub struct ConnectedMidiState {
    input_port: (MidiInputPort, String),
    output_port: (MidiOutputPort, String),
    input_connection: MidiInputConnection<MidiPassthroughListenerState>,
    event_rx: mpsc::Receiver<LiveEvent<'static>>,
}

impl ConnectedMidiState {
    pub fn input_port(&self) -> &(MidiInputPort, String) {
        &self.input_port
    }

    pub fn output_port(&self) -> &(MidiOutputPort, String) {
        &self.output_port
    }

    pub fn try_recv(&self) -> Option<LiveEvent<'static>> {
        self.event_rx.try_recv().ok()
    }

    pub fn disconnect(self) -> ReadyMidiState {
        let (input, passthrough_state) = self.input_connection.close();
        let output = passthrough_state.output_connection.close();
        ReadyMidiState { input, output }
    }
}

struct MidiPassthroughListenerState {
    output_connection: MidiOutputConnection,
    event_tx: mpsc::Sender<LiveEvent<'static>>,
}
