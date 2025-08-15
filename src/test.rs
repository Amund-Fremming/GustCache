#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::generate_hash;

    #[derive(Clone, Hash)]
    enum Category {
        One,
        Two,
        Three,
    }

    #[derive(Clone, Hash)]
    struct Request {
        num: i32,
        category: Category,
    }

    #[tokio::test]
    async fn unique_hash_generation() {
        let categories = Vec::from([Category::One, Category::Two, Category::Three]);
        let mut keys: HashMap<u64, bool> = HashMap::new();
        for category in categories {
            for num in 0..1_000_000 {
                let request = Request {
                    num,
                    category: category.clone(),
                };

                let key = generate_hash(&request);
                let value = keys.get(&key);

                assert!(value.is_none());
                keys.insert(key, true);
            }
        }
    }

    #[tokio::test]
    async fn same_key_same_data() {
        let request = Request {
            num: 1234,
            category: Category::One,
        };

        let key = generate_hash(&request);

        for _ in 0..1_000_000 {
            let cloned_request = request.clone();
            let cloned_key = generate_hash(&cloned_request);

            assert_eq!(key, cloned_key);

            let recreated_request = Request {
                num: request.num,
                category: request.category.clone(),
            };
            let recreated_key = generate_hash(&recreated_request);

            assert_eq!(key, recreated_key);
        }
    }
}
