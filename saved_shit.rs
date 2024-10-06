use indexmap::IndexMap;

// Assuming the definition of Profile and its key method
struct Profile {
    key: Key,
}

impl Profile {
    fn key(&self) -> &Key {
        &self.key
    }
}

struct Key {
    address: String,
}

impl Key {
    fn address(&self) -> &str {
        &self.address
    }
}

pub struct ProfileCollection(IndexMap<String, Profile>);

impl ProfileCollection {
    // Search for a Profile by address
    pub fn find_by_address(&self, address: &str) -> Option<&Profile> {
        self.0.values().find(|profile| profile.key().address() == address)
    }
}

fn main() {
    // Example setup
    let mut profiles = IndexMap::new();
    profiles.insert("profile1".to_string(), Profile { key: Key { address: "addr1".to_string() } });
    profiles.insert("profile2".to_string(), Profile { key: Key { address: "addr2".to_string() } });

    let profile_collection = ProfileCollection(profiles);

    // Search for a profile by address
    if let Some(profile) = profile_collection.find_by_address("addr1") {
        println!("Profile found with address: {}", profile.key().address());
    } else {
        println!("Profile not found.");
    }
}
