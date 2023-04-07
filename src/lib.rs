#[cfg(test)]
mod tests {
    use bc_crypto::sha256;

    #[test]
    fn it_works() {
        let input = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        let expected = "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1";
        let result = hex::encode(sha256(input.as_bytes()));
        assert_eq!(result, expected);
    }
}
