use actix_web::App;
use digital_voting::api::{protocol::UnparsedVote, server::vote};
use digital_voting::{Vote, VotingSystem};
// TODO This test will probably become deprecated in the near future.
#[test]
async fn test_voting_system_happy_path() {
    let mut votes1 = Vec::new();
    votes1.push(Vote::new(1, 0, chrono::Utc::now()));
    votes1.push(Vote::new(2, 0, chrono::Utc::now()));
    votes1.push(Vote::new(3, 1, chrono::Utc::now()));
    votes1.push(Vote::new(4, 0, chrono::Utc::now()));

    let mut votes2 = Vec::new();
    votes2.push(Vote::new(5, 0, chrono::Utc::now()));
    votes2.push(Vote::new(6, 0, chrono::Utc::now()));
    votes2.push(Vote::new(7, 1, chrono::Utc::now()));

    let voting_system = VotingSystem::new()
        .add_votes(votes1)
        .unwrap()
        .add_votes(votes2)
        .unwrap();

    voting_system.validate().unwrap();
    println!("Votes are valid");

    // TODO not sure about file ops on CI.
    let votes_tally = voting_system.tally_votes().unwrap();
    voting_system.save_to_file("votes.bin").unwrap();

    let loaded_voting_system = VotingSystem::load_from_file("votes.bin").unwrap();
    std::fs::remove_file("votes.bin").unwrap();
    loaded_voting_system.validate().unwrap();
    let loaded_votes_tally = loaded_voting_system.tally_votes().unwrap();

    assert_eq!(votes_tally, loaded_votes_tally);
}

// TODO This can be used for external testing: curl -X POST localhost:8080/vote -H "Content-Type: application/json" -d '{"pkey": "AAECAw==", "vote": 1, "timestamp": "2021-07-01T00:00:00Z", "signature": "BAUGBw=="}'
// TODO This unit test relies a bit too much on protocol.rs maybe? Not sure if that's an issue tho.
use actix_web::{dev::ServiceResponse, test};
#[actix_web::test]
async fn test_index_get() {
    let app = test::init_service(App::new().service(vote)).await;
    let payload = UnparsedVote {
        pkey: vec![0, 1, 2, 3],
        vote: 1,
        timestamp: chrono::Utc::now(),
        signature: vec![4, 5, 6, 7],
    };
    let req = test::TestRequest::post()
        .uri("/vote")
        .set_json(&payload)
        .to_request();
    let resp: ServiceResponse = test::call_service(&app, req).await;
    let received_payload: UnparsedVote = test::read_body_json(resp).await;
    assert_eq!(payload, received_payload);
}
