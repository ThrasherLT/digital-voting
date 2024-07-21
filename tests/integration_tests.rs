use digital_voting::{Vote, VotingSystem};

#[test]
fn test_something() {
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
