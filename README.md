# stochos
A stochastic music generation engine.

## Usage
Start the REPL with `cargo run`, then call functions at the `>` prompt.
Arguments are named and accept arithmetic expressions (`+ - * /`, parentheses).
Type `exit`, `quit`, or press Ctrl-D to leave.

```text
> bi_trial(p = 0.5)
1
> bi_trial(q=1/3)
0
```

## Supported stochastic sources:
- [x] [Bernoulli trial](#bernoulli-trial)

<a id="bernoulli-trial"></a>
### Bernoulli trial
A trial with two possible outcomes that can be defined with the $p + q = 1$ relation, where $p$ is the probability of success, and $q$ of failure.
#### Applications
- Describes whether the current event spans a note or rest
#### Parameters
- $p$: probability of success
- $q$: probability of failure
- If $p$ is defined, $q$ cannot be, and vice-versa
#### Usage
- `bi_trial(p = <prob>)` or `bi_trial(q = <prob>)`
- Returns `1` on success and `0` on failure
- Exactly one of `p` or `q` must be given, within $[0, 1]$
