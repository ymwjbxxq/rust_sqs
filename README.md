# Rust + SQS + Lambda + DynamoDB

I wanted to write something a bit more complex than hello world and still practising RUST with Lamba. So this time, a Lambda is triggered from SQS and based on the message I add to DynamoDB or send a message after a query to a different SQS.
Again, it is not special without much fantasy, but it is good to practise because there is something new to learn each time. 

### What I have Learnt ###


From my commits you can see three versions of the same code.
Concurrency:
Concurrency means that an application is making progress on more than one task at the same time (concurrently).

Parallelism:
Parallelism means that an application splits its tasks up into smaller subtasks that can be processed in parallel, for example, on multiple CPUs simultaneously.

**The first try:**
```
// convert array in streams to use async
let mut records = stream::iter(event.records); 
// processing one element at a time
while let Some(record) = records.next().await {
```
I decided that I wanted to process all the SQS messages in parallel.

**The second try:**
```
let mut tasks = Vec::with_capacity(event.records.len());
let shared_client = Arc::from(client.clone());
log::info!("records {:?}", records);
for record in event.records.into_iter() {
    let _shared_client = Arc::clone(&shared_client);
    tasks.push(task::spawn(async move {
          .....
    }
}))

task::block_on(async {
     for t in tasks {
         t.await; 
      }
});
```
I have used use async_std::task, but I got a strange error.
``` 
thread 'async-std/runtime' panicked at 'there is no reactor running, must be called from the context of a Tokio 1.x runtime'
```
Not sure what does it mean. Maybe the library is not compatible with Tokio. So I decided to switch to Tokio and have one less dependency.

Before finding the proper syntax, I  struggled with the
Borrowing in async thread-spawning fn parameters.

When you call .iter() on a Vec, the values the iterator produces are references to the values in the Vec - they're being borrowed. So you can't use them in a thread because the Rust lifetime system needs to know that if you borrow something, that will exist for the whole time. 
There are a few ways:

* If you don't need the Vec after processing, you could switch to use [.into_iter()](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html#tymethod.into_iter). This creates a consuming iterator, that is, one that moves each value out of the vector (from start to end). The vector cannot be used after calling this.
* If you do need your Vec after, and your values are clonable, you could call .iter().cloned() instead of .iter() - this will make copies of each of the values, which you can then move into the thread.
* [Arc<T>](https://doc.rust-lang.org/std/sync/struct.Arc.html) that is a thread-safe reference-counting pointer. 'Arc' stands for 'Atomically Reference Counted'. The type Arc<T> provides shared ownership of a value of type T, allocated in a heap. Invoking clone on Arc produces a new Arc instance, which points to the same allocation on the heap as the source Arc bu you still need .iter().cloned()

### TEST RESULTS ###

The lambda does two operations based on the sqs message (it should be two lambda functions, but for the sake of the test, I made it one)

* First case:  Add into DynamoDB
* Second case: Query DynamoDB and send it to a queue

I have tested both operations with 10K SQS messages:

* 128 MB
* 1024 MB
* 2048 MB
* 3072 MB

If we take this as a rough calculation price:

Total compute (seconds) = Invocations * Average ms =  seconds
Total compute (GB-s) = seconds * LambdaMemory/1024 MB = totalCompute GB-s
Total charges = totalCompute * 	Price per 1ms = $$

First case:
| Memory | Max       | Average  | Minimun  | Invocations | Cost on Average |
| ------ | ----------|----------|----------|-------------| ----------------|
| 128    | 206.69 ms | 18.46 ms | 4.21 ms  | 2.87k       | 0,0000139073025 |
| 1024   | 137.33 ms | 8.29 ms  | 4.74 ms  | 3.34k       | 0,00046239962   |
| 2048   | 258.24 ms | 11.14 ms | 5.28 ms  | 2.72k       | 0,00201803328   |
| 3072   | 248.04 ms | 8.12 ms  | 4.45 ms  | 2.55k       | 0,0031059       |

Second case:
| Memory | Max       | Average  | Minimun  | Invocations | Cost on Average |
| ------ | ----------|----------|----------|-------------| ----------------|
| 128    | 553.79 ms | 43.29 ms | 7.58 ms  | 2.74k       | 0,0000311363325 |
| 1024   | 275.16 ms | 16.16 ms | 8.07 ms  | 2.91k       | 0,00078532752   |
| 2048   | 264.49 ms | 16.84 ms | 7.85 ms  | 2.41k       | 0,00270292104   |
| 3072   | 220.01 ms | 15.52 ms | 7.36 ms  | 2.56k       | 0,00595968      |

As usual, the best deal is with 1024 MB, but each case could be different.

### 128 MB - ADD ###
![picture](https://github.com/ymwjbxxq/rust_sqs/blob/main/readme/add-128.png)

### 1024 MB - ADD ###
![picture](https://github.com/ymwjbxxq/rust_sqs/blob/main/readme/add-1024.png)

### 2048 MB - ADD ###
![picture](https://github.com/ymwjbxxq/rust_sqs/blob/main/readme/add-2048.png)

### 3072 MB - ADD ###
![picture](https://github.com/ymwjbxxq/rust_sqs/blob/main/readme/add-3072.png)

### 128 MB - READ and SEND ###
![picture](https://github.com/ymwjbxxq/rust_sqs/blob/main/readme/read_and_send-128.png)

### 1024 MB - READ and SEND ###
![picture](https://github.com/ymwjbxxq/rust_sqs/blob/main/readme/read_and_send-1024.png)

### 2048 MB - READ and SEND ###
![picture](https://github.com/ymwjbxxq/rust_sqs/blob/main/readme/read_and_send-2048.png)

### 3072 MB - READ and SEND ###
![picture](https://github.com/ymwjbxxq/rust_sqs/blob/main/readme/read_and_send-3072.png)

### Build ###
```
make build
```

### Deploy ###
```
make deploy
```

### Cleanup ###
```
make delete
```