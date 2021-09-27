// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_gossip::backstage::GossipActor;
use bee_logger::logger_init;
use bee_node::{
    banner::print_logo_and_version,
    cli::NodeCliArgs,
    config::{NodeConfigBuilder, DEFAULT_NODE_CONFIG_PATH},
};

use backstage::core::{
    AbortableUnboundedChannel, Actor, ActorError, ActorResult, EolEvent, ReportEvent, Rt, Runtime, ScopeId, Service,
    StreamExt, SupHandle,
};

use std::error::Error;

#[derive(Default)]
struct BeeSupervisor {}

impl BeeSupervisor {
    fn new() -> Self {
        Self::default()
    }
}

enum BeeSupervisorEvent {
    Eol,
    Report,
}

impl<T> EolEvent<T> for BeeSupervisorEvent {
    fn eol_event(_scope: ScopeId, _service: Service, _actor: T, _r: ActorResult<()>) -> Self {
        Self::Eol
    }
}

impl<T> ReportEvent<T> for BeeSupervisorEvent {
    fn report_event(_scope: ScopeId, _service: Service) -> Self {
        Self::Report
    }
}

#[async_trait::async_trait]
impl<S: SupHandle<Self>> Actor<S> for BeeSupervisor {
    type Data = ();
    type Channel = AbortableUnboundedChannel<BeeSupervisorEvent>;

    async fn init(&mut self, rt: &mut Rt<Self, S>) -> ActorResult<Self::Data> {
        let cli = NodeCliArgs::new();

        let config = NodeConfigBuilder::from_file(cli.config().unwrap_or(DEFAULT_NODE_CONFIG_PATH))
            .map_err(ActorError::aborted)?
            .with_cli_args(&cli)
            .finish();

        logger_init(config.logger).map_err(ActorError::aborted)?;

        print_logo_and_version();

        if cli.version() {
            return Ok(());
        }

        let identity_config = config.identity;
        let gossip_config = config.gossip;
        let manual_peering_config = config.manual_peering;
        let auto_peering_config = config.auto_peering;

        log::info!("Local Id: {:?}", identity_config.local_id);

        rt.add_resource(identity_config).await;
        rt.add_resource(gossip_config).await;
        rt.add_resource(manual_peering_config).await;
        rt.add_resource(auto_peering_config).await;

        rt.start(Some("network".into()), GossipActor::new()).await?;

        Ok(())
    }

    async fn run(&mut self, rt: &mut Rt<Self, S>, _data: Self::Data) -> ActorResult<()> {
        while let Some(event) = rt.inbox_mut().next().await {
            match event {
                BeeSupervisorEvent::Eol | BeeSupervisorEvent::Report => {
                    // TODO: handle network status report events.
                }
            }
        }

        Ok(())
    }

    fn type_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    Runtime::new(Some("root".into()), BeeSupervisor::new())
        .await?
        .block_on()
        .await?;

    Ok(())
}
