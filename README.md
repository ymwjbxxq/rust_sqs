# Rust + SQS + Lambda + DynamoDB

I wanted to write something a bit more complex than hello world and still practising RUST with Lamba. So this time, a Lambda is triggered from SQS and based on the message I add to DynamoDb or send a message after a query to a different SQS.
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


### Deploy ###
```
./deploy.sh
```
It will take care to build, create the zip and run for you the sam deploy.

### Cleanup ###
```
sam delete --stack-name STACK_NAME
```
