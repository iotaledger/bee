// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    startup().await.unwrap();
}

async fn startup() -> anyhow::Result<()> {
    std::panic::set_hook(Box::new(|info| {
        log::error!("{}", info);
    }));

    // RuntimeScope::<ActorRegistry>::launch(|scope| {
    //     async move {
    //         scope
    //             .spawn_system_unsupervised(Launcher, Arc::new(RwLock::new(LauncherAPI)))
    //             .await?;

    //         scope
    //             .spawn_task(|rt| {
    //                 async move {
    //                     for i in 0..10 {
    //                         rt.system::<Launcher>()
    //                             .await
    //                             .unwrap()
    //                             .state
    //                             .read()
    //                             .await
    //                             .send_to_hello_world(HelloWorldEvent::Print(format!("foo {}", i)), rt)
    //                             .await
    //                             .unwrap();

    //                         tokio::time::sleep(Duration::from_secs(1)).await;
    //                     }
    //                     Ok(())
    //                 }
    //                 .boxed()
    //             })
    //             .await;
    //         Ok(())
    //     }
    //     .boxed()
    // })
    // .await?;
    Ok(())
}
