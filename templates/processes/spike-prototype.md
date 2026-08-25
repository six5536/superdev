# Process: Spikes and prototypes

Code written to answer a question, not to keep. The discipline is keeping those two purposes separate.

## 1. Name the question

- A spike exists to resolve a specific uncertainty: "can library X do Y?", "is approach A fast enough?", "what does this API actually return?". Write the question down first.
- Define what answers it — the observation or number that ends the spike. Without this, a spike becomes an unplanned implementation.

## 2. Timebox and isolate

- Agree an effort bound up front; hitting the bound without an answer is itself a finding ("harder than expected") to report, not a license to continue silently.
- Keep spike code out of the real tree: scratchpad directory, a clearly-named throwaway branch, or a sandbox file — never mixed into production modules.

## 3. Cut every corner except the one being tested

- Hardcode inputs, skip error handling, ignore style — speed to answer is the goal. But be rigorous about the thing under test: a performance spike needs honest measurement; an API spike needs the real API, not a mock.
- Note assumptions made while cutting corners; they become caveats on the answer.

## 4. Extract the answer, not the code

- The deliverable is the finding: the answer, the evidence, the caveats, and a recommendation (see `templates/investigation.md`).
- Default: throw the code away. Production implementation restarts from the finding, following the normal feature process with tests and review.
- If any fragment is genuinely worth keeping, promote it deliberately — clean it to production standard, add tests, review it like new code. Never merge spike code because it "already works".

## 5. Report

- State clearly that this was a spike: what was learned, what was built only as scaffolding, and that the scaffolding is discarded (or what was promoted and how it was hardened).
