use super::*;

#[test]
fn correctness() {
    let transitions = vec![StorageTransition {
        address: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5],
        key: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5],
        value: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5],
        meta: None
    }];
    let compressed = StorageTransition::compress(transitions.clone());
    assert_eq!(transitions, StorageTransition::uncompress(compressed));
}

#[test]
fn effectivity_simple_contract() {
    let transitions = vec![StorageTransition {
        address: [221, 31, 123, 46, 34, 67, 213, 90, 55, 0, 12, 54, 222, 56, 77, 0, 132, 12, 1, 5],
        key: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 43],
        value: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 46, 123],
        meta: None
    }];
    let compressed = StorageTransition::compress(transitions.clone());
    let start_len = StorageTransition::into_bytes(transitions.clone()).len() as f64;
    let optimise = compressed.len() as f64;
    println!("Optimized {:.2} % for simple contract", (start_len-optimise)/start_len*100.0);
    assert_eq!(transitions, StorageTransition::uncompress(compressed));
}


#[test]
fn effectivity_erc20() {
    let transitions = vec![StorageTransition {
        address: [221, 31, 123, 46, 34, 67, 213, 90, 55, 0, 12, 54, 222, 56, 77, 0, 132, 12, 1, 5],
        key: [34, 1, 123, 23, 44, 65, 78, 66, 34, 0, 0, 0, 234, 0, 0, 243, 0, 0, 22, 0, 0, 0, 234, 0, 65, 0, 0, 0, 65, 0, 4, 43],
        value: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 21, 34, 46, 123],
        meta: None
    }, StorageTransition {
        address: [221, 31, 123, 46, 34, 67, 213, 90, 55, 0, 12, 54, 222, 56, 77, 0, 132, 12, 1, 5],
        key: [31, 8, 32, 23, 2, 65, 222, 1, 34, 0, 6, 0, 234, 0, 0, 243, 0, 0, 22, 0, 0, 0, 234, 0, 122, 65, 33, 0, 4, 0, 4, 11],
        value: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 222, 131, 5],
        meta: None
    }];
    let compressed = StorageTransition::compress(transitions.clone());
    let start_len = StorageTransition::into_bytes(transitions.clone()).len() as f64;
    let optimise = compressed.len() as f64;
    println!("Optimized {:.2} % for ERC20", (start_len-optimise)/start_len*100.0);
    assert_eq!(transitions, StorageTransition::uncompress(compressed));
}