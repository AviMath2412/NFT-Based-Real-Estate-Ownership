#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, Address, Vec, symbol_short};

// Struct for property details
#[contracttype]
#[derive(Clone)]
pub struct Property {
    pub property_id: u64,
    pub title: String,
    pub location: String,
    pub description: String,
    pub total_shares: u64,
    pub price_per_share: u64,
    pub image_url: String,
    pub registration_time: u64,
    pub is_verified: bool,
}

// Struct for tracking ownership shares
#[contracttype]
#[derive(Clone)]
pub struct OwnershipShare {
    pub property_id: u64,
    pub owner: Address,
    pub shares: u64,
    pub purchase_time: u64,
}

// Struct for tracking property statistics
#[contracttype]
#[derive(Clone)]
pub struct PropertyStats {
    pub total_properties: u64,
    pub verified_properties: u64,
    pub total_owners: u64,
    pub total_transactions: u64,
}

// Enum for mapping property IDs
#[contracttype] 
pub enum PropertyRegistry { 
    Property(u64)
}

// Enum for mapping ownership records
#[contracttype] 
pub enum OwnershipRegistry { 
    Ownership(u64, Address)
}

// Enum for user's owned properties
#[contracttype] 
pub enum UserProperties {
    Properties(Address)
}

// Constants for contract storage
const PROPERTY_COUNTER: Symbol = symbol_short!("PROP_CTR");
const PROPERTY_STATS: Symbol = symbol_short!("PROP_STAT");
const CONTRACT_ADMIN: Symbol = symbol_short!("ADMIN");

#[contract]
pub struct RealEstateNFT;

#[contractimpl]
impl RealEstateNFT {
    // Initialize the contract with an admin address
    pub fn initialize(env: Env, admin: Address) {
        // Ensure contract is only initialized once
        if env.storage().instance().has(&CONTRACT_ADMIN) {
            panic!("Contract already initialized");
        }
        
        // Store admin address
        env.storage().instance().set(&CONTRACT_ADMIN, &admin);
        
        // Initialize property stats
        let stats = PropertyStats {
            total_properties: 0,
            verified_properties: 0,
            total_owners: 0,
            total_transactions: 0,
        };
        
        env.storage().instance().set(&PROPERTY_STATS, &stats);
        env.storage().instance().set(&PROPERTY_COUNTER, &0u64);
        
        env.storage().instance().extend_ttl(10000, 10000);
        log!(&env, "RealEstateNFT contract initialized with admin: {}", admin);
    }
    
    // Function to register a new property
    pub fn register_property(
        env: Env, 
        title: String, 
        location: String, 
        description: String, 
        total_shares: u64, 
        price_per_share: u64,
        image_url: String
    ) -> u64 {
        // Get next property ID
        let mut property_counter: u64 = env.storage().instance().get(&PROPERTY_COUNTER).unwrap_or(0);
        property_counter += 1;
        
        // Get current timestamp
        let timestamp = env.ledger().timestamp();
        
        // Create new property
        let property = Property {
            property_id: property_counter,
            title,
            location,
            description,
            total_shares,
            price_per_share,
            image_url,
            registration_time: timestamp,
            is_verified: false,
        };
        
        // Update property stats
        let mut stats = Self::get_property_stats(env.clone());
        stats.total_properties += 1;
        
        // Store property data
        env.storage().instance().set(&PropertyRegistry::Property(property_counter), &property);
        env.storage().instance().set(&PROPERTY_COUNTER, &property_counter);
        env.storage().instance().set(&PROPERTY_STATS, &stats);
        
        env.storage().instance().extend_ttl(10000, 10000);
        log!(&env, "New property registered with ID: {}", property_counter);
        
        property_counter
    }
    
    // Function to verify a property (admin only)
    pub fn verify_property(env: Env, property_id: u64) {
        // Check admin authorization
        let admin: Address = env.storage().instance().get(&CONTRACT_ADMIN).expect("Contract not initialized");
        admin.require_auth();
        
        // Get property data
        let key = PropertyRegistry::Property(property_id);
        let mut property: Property = env.storage().instance().get(&key).expect("Property not found");
        
        // Check if property is already verified
        if property.is_verified {
            panic!("Property already verified");
        }
        
        // Update verification status
        property.is_verified = true;
        
        // Update property stats
        let mut stats = Self::get_property_stats(env.clone());
        stats.verified_properties += 1;
        
        // Store updated data
        env.storage().instance().set(&key, &property);
        env.storage().instance().set(&PROPERTY_STATS, &stats);
        
        env.storage().instance().extend_ttl(10000, 10000);
        log!(&env, "Property ID: {} is now verified", property_id);
    }
    
    // Function to purchase property shares
    pub fn purchase_shares(env: Env, property_id: u64, shares: u64, buyer: Address) {
        // Authentication
        buyer.require_auth();
        
        // Get property data
        let key = PropertyRegistry::Property(property_id);
        let property: Property = env.storage().instance().get(&key).expect("Property not found");
        
        // Check if property is verified
        if !property.is_verified {
            panic!("Cannot purchase shares of unverified property");
        }
        
        // Get current ownership if exists
        let ownership_key = OwnershipRegistry::Ownership(property_id, buyer.clone());
        let existing_ownership: Option<OwnershipShare> = env.storage().instance().get(&ownership_key);
        
        // Calculate total owned shares after purchase
        let mut new_shares = shares;
        let is_new_owner = existing_ownership.is_none();
        
        if let Some(existing) = existing_ownership {
            new_shares += existing.shares;
        }
        
        // Ensure there are enough shares available
        let current_timestamp = env.ledger().timestamp();
        let ownership_share = OwnershipShare {
            property_id,
            owner: buyer.clone(),
            shares: new_shares,
            purchase_time: current_timestamp,
        };
        
        // Update user's property list
        let user_properties_key = UserProperties::Properties(buyer.clone());
        let mut user_properties: Vec<u64> = env.storage().instance().get(&user_properties_key).unwrap_or(Vec::new(&env));
        
        if is_new_owner {
            user_properties.push_back(property_id);
            
            // Update owner stats if this is a new owner
            let mut stats = Self::get_property_stats(env.clone());
            stats.total_owners += 1;
            stats.total_transactions += 1;
            env.storage().instance().set(&PROPERTY_STATS, &stats);
        } else {
            // Just increment transaction count
            let mut stats = Self::get_property_stats(env.clone());
            stats.total_transactions += 1;
            env.storage().instance().set(&PROPERTY_STATS, &stats);
        }
        
        // Store updated data
        env.storage().instance().set(&ownership_key, &ownership_share);
        env.storage().instance().set(&user_properties_key, &user_properties);
        
        env.storage().instance().extend_ttl(10000, 10000);
        log!(&env, "Address {} purchased {} shares of property {}", buyer, shares, property_id);
    }
    
    // Function to transfer shares to another user
    pub fn transfer_shares(env: Env, property_id: u64, from: Address, to: Address, shares: u64) {
        // Authentication
        from.require_auth();
        
        // Get sender's current ownership
        let from_key = OwnershipRegistry::Ownership(property_id, from.clone());
        let mut from_ownership: OwnershipShare = env.storage().instance().get(&from_key)
            .expect("You don't own shares of this property");
        
        // Check if sender has enough shares
        if from_ownership.shares < shares {
            panic!("Insufficient shares to transfer");
        }
        
        // Update sender's shares
        from_ownership.shares -= shares;
        
        // Get recipient's current ownership
        let to_key = OwnershipRegistry::Ownership(property_id, to.clone());
        let current_timestamp = env.ledger().timestamp();
        
        let to_ownership: Option<OwnershipShare> = env.storage().instance().get(&to_key);
        let new_to_ownership: OwnershipShare;
        
        if let Some(mut existing) = to_ownership {
            // Update existing ownership
            new_to_ownership = OwnershipShare {
                property_id,
                owner: to.clone(),
                shares: existing.shares + shares,
                purchase_time: current_timestamp,
            };
        } else {
            // Create new ownership record for recipient
            new_to_ownership = OwnershipShare {
                property_id,
                owner: to.clone(),
                shares,
                purchase_time: current_timestamp,
            };
            
            // Add property to recipient's property list
            let to_properties_key = UserProperties::Properties(to.clone());
            let mut to_properties: Vec<u64> = env.storage().instance().get(&to_properties_key).unwrap_or(Vec::new(&env));
            to_properties.push_back(property_id);
            env.storage().instance().set(&to_properties_key, &to_properties);
            
            // Update owner stats if this is a new owner
            let mut stats = Self::get_property_stats(env.clone());
            stats.total_owners += 1;
            env.storage().instance().set(&PROPERTY_STATS, &stats);
        }
        
        // Update transaction count
        let mut stats = Self::get_property_stats(env.clone());
        stats.total_transactions += 1;
        env.storage().instance().set(&PROPERTY_STATS, &stats);
        
        // Store updated ownership data
        env.storage().instance().set(&from_key, &from_ownership);
        env.storage().instance().set(&to_key, &new_to_ownership);
        
        env.storage().instance().extend_ttl(10000, 10000);
        log!(&env, "{} transferred {} shares of property {} to {}", from, shares, property_id, to);
    }
    
    // View function to get property details
    pub fn get_property(env: Env, property_id: u64) -> Property {
        let key = PropertyRegistry::Property(property_id);
        env.storage().instance().get(&key).expect("Property not found")
    }
    
    // View function to get ownership details
    pub fn get_ownership(env: Env, property_id: u64, owner: Address) -> OwnershipShare {
        let key = OwnershipRegistry::Ownership(property_id, owner.clone());
        env.storage().instance().get(&key).unwrap_or(OwnershipShare {
            property_id,
            owner: owner.clone(),
            shares: 0,
            purchase_time: 0,
        })
    }
    
    // View function to get properties owned by an address
    pub fn get_user_properties(env: Env, owner: Address) -> Vec<u64> {
        let key = UserProperties::Properties(owner);
        env.storage().instance().get(&key).unwrap_or(Vec::new(&env))
    }
    
    // View function to get property statistics
    pub fn get_property_stats(env: Env) -> PropertyStats {
        env.storage().instance().get(&PROPERTY_STATS).unwrap_or(PropertyStats {
            total_properties: 0,
            verified_properties: 0,
            total_owners: 0,
            total_transactions: 0,
        })
    }
    
    // Function to get total shares owned across all properties by an address
    pub fn get_total_shares_owned(env: Env, owner: Address) -> u64 {
        let properties = Self::get_user_properties(env.clone(), owner.clone());
        let mut total_shares = 0;
        
        for property_id in properties.iter() {
            let ownership = Self::get_ownership(env.clone(), property_id, owner.clone());
            total_shares += ownership.shares;
        }
        
        total_shares
    }
    
    // Function to list all properties with pagination
    pub fn list_properties(env: Env, start_idx: u64, limit: u64) -> Vec<Property> {
        let property_counter: u64 = env.storage().instance().get(&PROPERTY_COUNTER).unwrap_or(0);
        let mut properties = Vec::new(&env);
        
        let end_idx = if start_idx + limit > property_counter {
            property_counter
        } else {
            start_idx + limit
        };
        
        for i in start_idx..=end_idx {
            if i > 0 {
                let key = PropertyRegistry::Property(i);
                if let Some(property) = env.storage().instance().get::<PropertyRegistry, Property>(&key) {
                    properties.push_back(property);
                }
            }
        }
        
        properties
    }
}