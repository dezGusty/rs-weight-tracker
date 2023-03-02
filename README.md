# rs-weight-tracker

A Rust based weight tracker, implemented with the help of AI assistance (E.g. ChatGPT)

## Goal

Create an application written in Rust to manage weights (add/import/export/display).
There is no specific end to the features to implement.

## Prerequisites

Unfortunately it is not (yet) possible to have ChatGPT write a full application in Rust without understanding and correcting the output it produces.

I went through Microsoft's tutorial on Rust a while back. You can find it here: <https://learn.microsoft.com/en-us/training/paths/rust-first-steps/>

I had forgotten many things, so I had to also re-read some chapters of the book: <https://doc.rust-lang.org/book/>

So the prerequisites are:

- know at least some basics of the Rust language
- have a development environment based on Rust set-up already (E.g. <https://www.rust-lang.org/learn/get-started>)

## Project set-up

The initial project set-up operation is described here: [step-by-step](./docs/step-by-step.md)
There may be some missing steps about the SQLite set-up; will have to re-visit.

## Sessions

### Session 1

Covers:

- adding migrations to the DB
- writing operation for adding weights
- writing operation for displaying weights
- writing operation for importing many weights from a json file

Can be found here: [session-1](./docs/session-1.md)

### Session 2

Covers:

- change DB structure and add another migration to the DB
- writing operation for displaying weights between a start- and end-date
- add update/insert function for weights (upsert)

Can be found here: [session-2](./docs/session-2.md)

### Session 3

Covers:

- add function for obtaining weights in an interval with interpolation of missing values.
- add function for obtaining rolling averages of weights in a given interval.

Can be found here: [session-3](./docs/session-3.md)

### Session 4

Covers:

- add web server using Rocket.
- add an endpoint to obtain data for rolling averages of weights in a given interval in a string format.
- update the endpoint to return json
- add a very basic UI calling the endpoint.
- add a chart making use endpoint results.

Can be found here: [session-4](./docs/session-4.md)

### Session 5

- fix issue with Chart (re)creation in front-end.
- add weight functionality (POST on back-end, full page in front-end)
- refactoring, removing use of deprecated functions

Can be found here: [session-5](./docs/session-5.md)

## Setting up locally

You will need the prerequisites.

And run

```cmd
diesel migration run
```
