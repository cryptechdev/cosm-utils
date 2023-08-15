pub fn find_event_tag(events: &mut std::slice::Iter<'_, cosmrs::tendermint::abci::Event>, event_type: String, key_name: String) -> Option<String> {
    events.find_map(|event| {
        if event.kind == event_type {
            event.attributes
            .iter()
            .find_map(|attr| { 
                if attr.key == key_name {
                    Some(attr.value.clone())
                } else {
                    None
                }
            })
        } else {
            None
        }
    })
}