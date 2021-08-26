async fn wip() {

    let mission_registry = MissionRegistryTwin {};
    let mission_registry_1_state = twin::execute_twin(service.clone(), mission_registry)?;

    let mission_registry = tokio::spawn(launchpad_1_state.map(move |state| {
        println!("{:?}", state);
    }));

    match launchpad_1_state.next().await {
        Ok(state) => {
            println!("{:?}", state);
            if let Some(mission) = state.mission {
                println!("{:?}", mission);
                match *(twin_current_state(service.clone(), MissionTwin { id: mission }).await)
                {
                    Ok(mission_state) => println!("{:?}", mission_state),
                    Err(e) => println!("{:?}", e),
                }
            }
        }
        _ => (),
    }
    match mission_registry_1_state.try_recv() {
        Ok(state) => {
            println!("{:?}", state);
        }
        _ => (),
    }
    std::thread::sleep(Duration::from_secs_f32(0.33f32));
}

async fn observe_twin() {
    let launchpad_thread = observe(
        twin::execute_twin(
            service.clone(),
            LaunchpadTwin {
                id: "Launchpad-01".to_string(),
            },
        ),
        |state| println!("launchpad state {:?}", state),
    );
    let _ = join!(launchpad_thread);
}

async fn observe() {
    let missions_thread = observe(
        twin::execute_twin(service.clone(), MissionRegistryTwin),
        |state| println!("Missions state {:?}", state),
    );
    let _ = join!(missions_thread);

}

async fn test_switch_map_combine_latest () {
    let mut current_mission = switch_map(
        twin::execute_twin(service.clone(), MissionRegistryTwin),
        |state| {
            Some(combine_latest(
                state
                    .into_iter()
                    .map(|id| twin::execute_twin(service.clone(), MissionTwin { id }))
                    .collect(),
            ))
        },
    );

    observe(
        current_mission,
        |states| println!("all_mission {:?}", states),
    );
}

async fn test_observe_registry () {
    let current_mission = observe_registry(
        service.clone(),
        MissionRegistryTwin,
        |s| s.into_iter().map(|id| MissionTwin { id }).collect(),
        |states| println!("all_mission {:?}", states),
    );
}