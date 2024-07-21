// use digital_voting::Error as BlockchainError;
use digital_voting::Vote;
use digital_voting::VotingSystem;
use rand::Rng;

fn main() -> Result<(), anyhow::Error> {
    let mut rng = rand::thread_rng();
    let mut votes1 = Vec::new();
    for i in 1..10 {
        votes1.push(Vote::new(i, rng.gen_range(0..=1), chrono::Utc::now()));
    }

    let mut votes2 = Vec::new();
    for i in 1..10 {
        votes2.push(Vote::new(i, rng.gen_range(0..=1), chrono::Utc::now()));
    }

    let voting_system = VotingSystem::new().add_votes(votes1)?.add_votes(votes2)?;
    println!("{}", voting_system);

    voting_system.validate()?;
    println!("Votes are valid");

    println!("{}", voting_system.tally_votes().unwrap());

    voting_system.save_to_file("votes.bin")?;
    let loaded_voting_system = VotingSystem::load_from_file("votes.bin")?;

    println!("{}", loaded_voting_system);

    loaded_voting_system.validate()?;
    println!("Votes are still valid");

    println!("{}", loaded_voting_system.tally_votes().unwrap());

    Ok(())
}
