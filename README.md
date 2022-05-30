### toy

I believe I handled all cases. The one area I wasn't clear on is whether withdrawal transactions can be disputed and charged back. So I assumed yes and implemented the logic accordingly.

I did add quite a few tests that can be run with ```cargo test``` to ensure things behave as requested.

As for efficiency. The app uses memory to store the accounts and transactions.

This was a tradeoff as I assumed that would be sufficient for this task, for a datasets larger than a million of transactions (or exceeding available memory whichever comes first) 
an actual DB would be used to store the accounts and transactions. 

Last I opted to use tokio runtime since the description was mentioning accepting inputs from several csv files, and mpsc channel seemed like a natural fit, for such task, and there is a unittest with multiple consumers that demonstrates the scenario.
