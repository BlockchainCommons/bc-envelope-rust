# Expected Text Output Rubric

Don't use a bunch of `assert` calls on a serialized or deserialized structure like an `Envelope`. Instead, use one pass to log the expected text (or dummy text), then run the test, let it fail, then copy the correct output text into code using `indoc`, and then use a single `assert(text).toBe(expectedText)` call to compare the actual output to the expected output. This can replace many brittle expect calls with a single one, making it easier to maintain tests and understand failures. The codebase contains many examples of this, as well as utilities like `expectOutput` that facilitate the rubric. You *don't* need to use `dedent` for short strings.

## Example: Envelope serialization

Frequently a helper should be used to do the round-trip serialization and deserialization, and test against expected text serialization or if not provided, print the expected text serialization to the console so it can be collected for later use.

### Phase One: Collect expected envelope notation

The test is run without expected text output first, for example:

```rust
    println!("{}", envelope.format());
```

which outputs:

```
response(ARID(c66be27d)) [
    'result': 'OK'
]
```

### Phase Two: Test against previous output

Copy the expected text output back into the test and run the test to confirm it works:

```rust
    #[rustfmt::skip]
    assert_eq!(envelope.format(), (indoc! {r#"
        response(ARID(c66be27d)) [
            'result': 'OK'
        ]
    "#}).trim());
```

Note that the `#[rustfmt::skip]` attribute is used to prevent rustfmt from reformatting the indented string literal, and the `trim()` method is called to remove any leading or trailing whitespace from the expected output. Note that the expected output is indented one more level, and is kept separate from the code around it.
