#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Env, Address, Vec, String, Symbol, symbol_short};

// Storage keys for the contract
const FACT_COUNT: Symbol = symbol_short!("FACT_CNT");
const FACT_PREFIX: Symbol = symbol_short!("FACT");

/// Represents a single fact with voting data
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fact {
    pub id: u32,
    pub text: String,
    pub creator: Address,
    pub true_votes: u32,
    pub false_votes: u32,
    pub voters: Vec<Address>,
}

#[contract]
pub struct FactVerificationContract;

#[contractimpl]
impl FactVerificationContract {
    /// Submit a new fact to the blockchain
    /// 
    /// # Arguments
    /// * env - The Soroban environment
    /// * creator - The address of the fact creator
    /// * text - The fact text content
    /// 
    /// # Returns
    /// * u32 - The ID of the newly created fact
    pub fn submit_fact(env: Env, creator: Address, text: String) -> u32 {
        // Require authorization from the creator
        creator.require_auth();

        // Get the current fact count (or initialize to 0)
        let fact_count: u32 = env.storage().instance().get(&FACT_COUNT).unwrap_or(0);
        
        // Create new fact ID (starting from 1)
        let new_id = fact_count + 1;

        // Create the fact struct
        let fact = Fact {
            id: new_id,
            text: text.clone(),
            creator: creator.clone(),
            true_votes: 0,
            false_votes: 0,
            voters: Vec::new(&env),
        };

        // Store the fact using a composite key
        let fact_key = (FACT_PREFIX, new_id);
        env.storage().persistent().set(&fact_key, &fact);

        // Update the fact count
        env.storage().instance().set(&FACT_COUNT, &new_id);

        // Extend the TTL for the fact and counter
        env.storage().persistent().extend_ttl(&fact_key, 100, 100);
        env.storage().instance().extend_ttl(100, 100);

        new_id
    }

    /// Vote on a fact (true or false)
    /// 
    /// # Arguments
    /// * env - The Soroban environment
    /// * voter - The address of the voter
    /// * fact_id - The ID of the fact to vote on
    /// * is_true - True for "true" vote, false for "false" vote
    /// 
    /// # Panics
    /// * If the fact doesn't exist
    /// * If the voter has already voted on this fact
    pub fn vote(env: Env, voter: Address, fact_id: u32, is_true: bool) {
        // Require authorization from the voter
        voter.require_auth();

        // Retrieve the fact
        let fact_key = (FACT_PREFIX, fact_id);
        let mut fact: Fact = env.storage()
            .persistent()
            .get(&fact_key)
            .expect("Fact not found");

        // Check if voter has already voted
        for existing_voter in fact.voters.iter() {
            if existing_voter == voter {
                panic!("Already voted on this fact");
            }
        }

        // Add voter to the list
        fact.voters.push_back(voter.clone());

        // Increment the appropriate vote counter
        if is_true {
            fact.true_votes += 1;
        } else {
            fact.false_votes += 1;
        }

        // Save the updated fact
        env.storage().persistent().set(&fact_key, &fact);
        
        // Extend TTL
        env.storage().persistent().extend_ttl(&fact_key, 100, 100);
    }

    /// Get details of a specific fact
    /// 
    /// # Arguments
    /// * env - The Soroban environment
    /// * fact_id - The ID of the fact to retrieve
    /// 
    /// # Returns
    /// * Fact - The fact details
    /// 
    /// # Panics
    /// * If the fact doesn't exist
    pub fn get_fact(env: Env, fact_id: u32) -> Fact {
        let fact_key = (FACT_PREFIX, fact_id);
        let fact: Fact = env.storage()
            .persistent()
            .get(&fact_key)
            .expect("Fact not found");
        
        // Extend TTL on read
        env.storage().persistent().extend_ttl(&fact_key, 100, 100);
        
        fact
    }

    /// Get all facts stored in the contract
    /// 
    /// # Arguments
    /// * env - The Soroban environment
    /// 
    /// # Returns
    /// * Vec<Fact> - A vector containing all facts
    pub fn get_all_facts(env: Env) -> Vec<Fact> {
        let fact_count: u32 = env.storage().instance().get(&FACT_COUNT).unwrap_or(0);
        let mut facts = Vec::new(&env);

        // Iterate through all fact IDs and collect them
        for id in 1..=fact_count {
            let fact_key = (FACT_PREFIX, id);
            if let Some(fact) = env.storage().persistent().get::<_, Fact>(&fact_key) {
                facts.push_back(fact);
                // Extend TTL
                env.storage().persistent().extend_ttl(&fact_key, 100, 100);
            }
        }

        facts
    }

    /// Get the total number of facts
    /// 
    /// # Arguments
    /// * env - The Soroban environment
    /// 
    /// # Returns
    /// * u32 - The total count of facts
    pub fn get_fact_count(env: Env) -> u32 {
        env.storage().instance().get(&FACT_COUNT).unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_submit_fact() {
        let env = Env::default();
        let contract_id = env.register_contract(None, FactVerificationContract);
        let client = FactVerificationContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let fact_text = String::from_str(&env, "The Earth is round");

        // Mock the authorization
        env.mock_all_auths();

        // Submit a fact
        let fact_id = client.submit_fact(&creator, &fact_text);

        assert_eq!(fact_id, 1);

        // Verify the fact was stored correctly
        let fact = client.get_fact(&fact_id);
        assert_eq!(fact.id, 1);
        assert_eq!(fact.text, fact_text);
        assert_eq!(fact.creator, creator);
        assert_eq!(fact.true_votes, 0);
        assert_eq!(fact.false_votes, 0);
        assert_eq!(fact.voters.len(), 0);
    }

    #[test]
    fn test_vote_true() {
        let env = Env::default();
        let contract_id = env.register_contract(None, FactVerificationContract);
        let client = FactVerificationContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let voter = Address::generate(&env);
        let fact_text = String::from_str(&env, "Water freezes at 0°C");

        env.mock_all_auths();

        // Submit a fact
        let fact_id = client.submit_fact(&creator, &fact_text);

        // Vote true
        client.vote(&voter, &fact_id, &true);

        // Check the vote was recorded
        let fact = client.get_fact(&fact_id);
        assert_eq!(fact.true_votes, 1);
        assert_eq!(fact.false_votes, 0);
        assert_eq!(fact.voters.len(), 1);
        assert_eq!(fact.voters.get(0).unwrap(), voter);
    }

    #[test]
    fn test_vote_false() {
        let env = Env::default();
        let contract_id = env.register_contract(None, FactVerificationContract);
        let client = FactVerificationContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let voter = Address::generate(&env);
        let fact_text = String::from_str(&env, "The Moon is made of cheese");

        env.mock_all_auths();

        // Submit a fact
        let fact_id = client.submit_fact(&creator, &fact_text);

        // Vote false
        client.vote(&voter, &fact_id, &false);

        // Check the vote was recorded
        let fact = client.get_fact(&fact_id);
        assert_eq!(fact.true_votes, 0);
        assert_eq!(fact.false_votes, 1);
        assert_eq!(fact.voters.len(), 1);
    }

    #[test]
    #[should_panic(expected = "Already voted on this fact")]
    fn test_prevent_double_voting() {
        let env = Env::default();
        let contract_id = env.register_contract(None, FactVerificationContract);
        let client = FactVerificationContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let voter = Address::generate(&env);
        let fact_text = String::from_str(&env, "Rust is awesome");

        env.mock_all_auths();

        // Submit a fact
        let fact_id = client.submit_fact(&creator, &fact_text);

        // First vote
        client.vote(&voter, &fact_id, &true);

        // Second vote from same address - should panic
        client.vote(&voter, &fact_id, &false);
    }

    #[test]
    fn test_multiple_voters() {
        let env = Env::default();
        let contract_id = env.register_contract(None, FactVerificationContract);
        let client = FactVerificationContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let voter1 = Address::generate(&env);
        let voter2 = Address::generate(&env);
        let voter3 = Address::generate(&env);
        let fact_text = String::from_str(&env, "Blockchain is decentralized");

        env.mock_all_auths();

        // Submit a fact
        let fact_id = client.submit_fact(&creator, &fact_text);

        // Multiple voters
        client.vote(&voter1, &fact_id, &true);
        client.vote(&voter2, &fact_id, &true);
        client.vote(&voter3, &fact_id, &false);

        // Check votes
        let fact = client.get_fact(&fact_id);
        assert_eq!(fact.true_votes, 2);
        assert_eq!(fact.false_votes, 1);
        assert_eq!(fact.voters.len(), 3);
    }

    #[test]
    fn test_get_all_facts() {
        let env = Env::default();
        let contract_id = env.register_contract(None, FactVerificationContract);
        let client = FactVerificationContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        let fact1 = String::from_str(&env, "Fact 1");
        let fact2 = String::from_str(&env, "Fact 2");
        let fact3 = String::from_str(&env, "Fact 3");

        env.mock_all_auths();

        // Submit multiple facts
        client.submit_fact(&creator, &fact1);
        client.submit_fact(&creator, &fact2);
        client.submit_fact(&creator, &fact3);

        // Get all facts
        let all_facts = client.get_all_facts();
        assert_eq!(all_facts.len(), 3);
        assert_eq!(all_facts.get(0).unwrap().text, fact1);
        assert_eq!(all_facts.get(1).unwrap().text, fact2);
        assert_eq!(all_facts.get(2).unwrap().text, fact3);
    }

    #[test]
    fn test_get_fact_count() {
        let env = Env::default();
        let contract_id = env.register_contract(None, FactVerificationContract);
        let client = FactVerificationContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        env.mock_all_auths();

        // Initially 0
        assert_eq!(client.get_fact_count(), 0);

        // After one submission
        client.submit_fact(&creator, &String::from_str(&env, "Fact 1"));
        assert_eq!(client.get_fact_count(), 1);

        // After two submissions
        client.submit_fact(&creator, &String::from_str(&env, "Fact 2"));
        assert_eq!(client.get_fact_count(), 2);
    }
}